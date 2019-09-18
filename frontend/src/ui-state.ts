import { useContext, createContext } from "react"
import { observable } from "mobx"

class UIState {
    @observable listing: "library" | { playlistId: string } = "library"
}

const uiState = new UIState()
export const UIStateContext = createContext(uiState)
export const useUI = () => useContext(UIStateContext)
