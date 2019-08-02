interface Callback {
    onSuccess: (value: Payload) => void
    onFailure: (error: Payload) => void
}

export type Payload = {} | null

export class RPCWebSocket {
    private lastId = 0
    private requestsPendingOpen: [Payload, Callback][] = []
    private outstanding: { [id: string]: Callback } = {}

    private ws: WebSocket
    private eventHandler: (payload: Payload) => void

    constructor(url: string, eventHandler: (payload: Payload) => void = () => {}) {
        this.ws = new WebSocket(url)
        this.ws.onopen = this.onOpen
        this.ws.onmessage = this.onMessage
        this.eventHandler = eventHandler
    }

    private onOpen = () => {
        for (let i = this.requestsPendingOpen.length - 1; i >= 0; --i) {
            const [payload, callback] = this.requestsPendingOpen[i]
            this.send(payload, callback)
        }
        this.requestsPendingOpen = []
    }

    private onMessage = (event: MessageEvent) => {
        const payload = event.data
        const parsed = JSON.parse(payload)
        const id: string = parsed[0]
        if (id === "") {
            this.eventHandler(parsed[1])
        } else {
            const error: Payload | undefined = parsed[2]
            // TODO: `id` might not be in `outstanding`
            if (error === undefined) {
                this.outstanding[id].onSuccess(parsed[1])
            } else {
                // TODO: more sensible error reporting
                this.outstanding[id].onFailure(new Error("Server error: " + JSON.stringify(error)))
            }
            delete this.outstanding[id]
        }
    }

    private send(payload: Payload, callback: Callback) {
        // TODO: increment id string directly to avoid large number problems
        const id = ++this.lastId
        const stringId = id.toString(36)
        this.outstanding[stringId] = callback
        this.ws.send(JSON.stringify([stringId, payload]))
    }

    query(payload: Payload) {
        switch (this.ws.readyState) {
            case WebSocket.OPEN:
                return new Promise<Payload>((resolve, reject) => {
                    return this.send(payload, { onSuccess: resolve, onFailure: reject })
                })
            case WebSocket.CONNECTING:
                return new Promise<Payload>((resolve, reject) => {
                    this.requestsPendingOpen.push([payload, { onSuccess: resolve, onFailure: reject }])
                })
            case WebSocket.CLOSING:
                return Promise.reject(new Error("Websocket closing"))
            case WebSocket.CLOSED:
                return Promise.reject(new Error("Websocket closed"))
        }
    }
}
