import { invoke } from '@tauri-apps/api/core'
import { create } from 'zustand'

interface ChainItem {
  id: string
  name: string
  vendor: string
  format: string
  bypassed: boolean
  unique_id?: string
}

interface ChainStore {
  chain: ChainItem[]
  loading: boolean
  loadingMessage: string | null
  setChain: (c: ChainItem[]) => void
  setLoading: (loading: boolean, message?: string | null) => void
  refresh: () => Promise<void>
  remove: (id: string) => void
}

export const useChainStore = create<ChainStore>(set => ({
  chain: [],
  loading: false,
  loadingMessage: null,
  setChain: chain => set({ chain }),
  setLoading: (loading, message = null) => set({ loading, loadingMessage: message }),
  refresh: async () => {
    try {
      const result = await invoke<ChainItem[]>('get_chain')
      set({ chain: result })
    }
    catch {
      // backend not ready
    }
  },
  remove: id =>
    set(s => ({ chain: s.chain.filter(p => p.id !== id) })),
}))

export type { ChainItem }
