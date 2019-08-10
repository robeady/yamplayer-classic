import { RPCWebSocket, Payload } from "../websocket"
import { Track } from "../Model"

interface ServerRPCApi {
    (type: "Enqueue", args: { track_id: string }): Promise<void>
    (type: "Stop"): Promise<void>
    (type: "TogglePause"): Promise<void>
    (type: "ChangeVolume", args: { muted?: boolean; volume?: number }): Promise<void>
    (type: "CompleteFilePath", args: { prefix: string }): Promise<void>
    (type: "GetLibrary"): Promise<{ tracks: Track[] }>
    (type: "AddToLibrary", args: { path: string }): Promise<void>
    (type: "GetPlaybackState"): Promise<{ playing: boolean; volume: number }>
}

type ServerEventHandler = (e: ServerEvent) => void

export type ServerEvent =
    | { type: "VolumeChanged"; args: { muted: boolean; volume: number } }
    | { type: "PlaybackPaused" }
    | { type: "PlaybackResumed" }
    | { type: "TrackChanged"; args: { track: Track } }

export class ServerApi {
    private handleEvent = (payload: Payload) => {
        this.handlers.forEach(handle => handle(payload as ServerEvent))
    }

    private ws = new RPCWebSocket("ws://127.0.0.1:8080/ws", this.handleEvent)
    private handlers: ServerEventHandler[] = []

    addHandler = (handler: ServerEventHandler) => {
        this.handlers.push(handler)
    }

    request: ServerRPCApi = async (type: string, args?: {}) => {
        return (await this.ws.query({ type, args })) as any
    }
}
