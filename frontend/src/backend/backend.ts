import { createContext, useContext } from "react"
import { Playback } from "./Playback"
import { ServerApi } from "./ServerApi"
import { Library } from "./Library"

const serverApi = new ServerApi()
export const BackendContext = createContext({
    playback: new Playback(serverApi),
    library: new Library(serverApi),
})
export const useBackend = () => useContext(BackendContext)
