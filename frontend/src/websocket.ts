interface Callback {
    onSuccess: (value: {} | null) => void
    onFailure: (error: {} | null) => void
}

type Payload = {} | null

export class RPCWebSocket {
    private lastId = 0
    private requestsAwaitingOpen: [Payload, Callback][] = []
    private outstanding = new Map<string, Callback>()
    private ws: WebSocket
    private eventHandler: (payload: {} | null) => void

    constructor(url: string, eventHandler: (payload: {} | null) => void = () => {}) {
        this.ws = new WebSocket(url)
        this.ws.onopen = this.onOpen
        this.ws.onmessage = this.onMessage
        this.eventHandler = eventHandler
    }

    private onOpen = () => {
        for (let i = this.requestsAwaitingOpen.length - 1; i >= 0; --i) {
            const [payload, callback] = this.requestsAwaitingOpen[i]
            this.send(payload, callback)
        }
        this.requestsAwaitingOpen = []
    }

    private onMessage = (event: MessageEvent) => {
        const payload = event.data
        const parsed = JSON.parse(payload)
        const id: string = parsed[0]
        if (id === "") {
            this.eventHandler(parsed[1])
        } else {
            const error: {} | null | undefined = parsed[2]
            // TODO: `id` might not be in `outstanding`
            if (error === undefined) {
                this.outstanding.get(id)!.onSuccess(parsed[1])
            } else {
                // TODO: more sensible error reporting
                this.outstanding.get(id)!.onFailure(new Error("Server error: " + JSON.stringify(error)))
            }
            this.outstanding.delete(id)
        }
    }

    private send(payload: {} | null, callback: Callback) {
        // TODO: increment id string directly to avoid large number problems
        const id = ++this.lastId
        const stringId = id.toString(36)
        this.outstanding.set(stringId, callback)
        this.ws.send(JSON.stringify([stringId, payload]))
    }

    query(payload: {} | null) {
        switch (this.ws.readyState) {
            case WebSocket.OPEN:
                return new Promise<Payload>((resolve, reject) => {
                    return this.send(payload, { onSuccess: resolve, onFailure: reject })
                })
            case WebSocket.CONNECTING:
                return new Promise<Payload>((resolve, reject) => {
                    this.requestsAwaitingOpen.push([payload, { onSuccess: resolve, onFailure: reject }])
                })
            case WebSocket.CLOSING:
                return Promise.reject(new Error("Websocket closing"))
            case WebSocket.CLOSED:
                return Promise.reject(new Error("Websocket closed"))
        }
    }
}
