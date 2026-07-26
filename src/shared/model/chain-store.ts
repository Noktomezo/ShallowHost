import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { create } from 'zustand'

export interface ChainItem {
  id: string
  name: string
  vendor: string
  format: string
  bypassed: boolean
  unique_id?: string
  initializing?: boolean
}

interface ChainStore {
  rawChain: ChainItem[]
  initializingMap: Record<string, ChainItem>
  chain: ChainItem[]
  error: string | null
  setError: (e: string | null) => void
  setChain: (c: ChainItem[]) => void
  refresh: () => Promise<void>
  loadPlaceholders: () => Promise<void>
  addPluginAsync: (plugin: { unique_id: string, name: string, vendor: string, format: string }) => void
  remove: (id: string) => void
  isInitializing: (uniqueId: string) => boolean
}

function computeChain(rawChain: ChainItem[], initializingMap: Record<string, ChainItem>): ChainItem[] {
  const pending = Object.values(initializingMap).filter(
    item => !rawChain.some(c => c.unique_id === item.unique_id || c.name === item.name),
  )
  return [...rawChain, ...pending]
}

export const useChainStore = create<ChainStore>((set, get) => ({
  rawChain: [],
  initializingMap: {},
  chain: [],
  error: null,
  setError: error => set({ error }),
  setChain: rawChain =>
    set(s => ({
      rawChain,
      chain: computeChain(rawChain, s.initializingMap),
    })),
  loadPlaceholders: async () => {
    try {
      const placeholders = await invoke<ChainItem[]>('get_saved_chain_placeholders')
      if (placeholders.length > 0) {
        const nextMap: Record<string, ChainItem> = {}
        for (const item of placeholders) {
          if (item.unique_id) {
            nextMap[item.unique_id] = { ...item, initializing: true }
          }
        }
        set(s => ({
          initializingMap: { ...nextMap, ...s.initializingMap },
          chain: computeChain(s.rawChain, { ...nextMap, ...s.initializingMap }),
        }))
      }
    }
    catch {
      // ignore
    }
  },
  refresh: async () => {
    try {
      const result = await invoke<ChainItem[]>('get_chain')
      set((s) => {
        const nextMap = { ...s.initializingMap }
        for (const item of result) {
          if (item.unique_id && nextMap[item.unique_id]) {
            delete nextMap[item.unique_id]
          }
          if (item.name && nextMap[item.name]) {
            delete nextMap[item.name]
          }
        }
        return {
          rawChain: result,
          initializingMap: nextMap,
          chain: computeChain(result, nextMap),
        }
      })
    }
    catch {
      // backend not ready
    }
  },
  addPluginAsync: (plugin) => {
    const tempItem: ChainItem = {
      id: `temp-${plugin.unique_id}`,
      name: plugin.name,
      vendor: plugin.vendor,
      format: plugin.format,
      bypassed: false,
      unique_id: plugin.unique_id,
      initializing: true,
    }

    set((s) => {
      const nextMap = { ...s.initializingMap, [plugin.unique_id]: tempItem }
      return {
        initializingMap: nextMap,
        chain: computeChain(s.rawChain, nextMap),
        error: null,
      }
    })

    invoke('add_to_chain', { pluginId: plugin.unique_id })
      .then(async () => {
        await get().refresh()
      })
      .catch((e) => {
        set({ error: String(e) })
      })
      .finally(() => {
        set((s) => {
          const nextMap = { ...s.initializingMap }
          delete nextMap[plugin.unique_id]
          return {
            initializingMap: nextMap,
            chain: computeChain(s.rawChain, nextMap),
          }
        })
      })
  },
  remove: (id) => {
    set(s => ({
      rawChain: s.rawChain.filter(p => p.id !== id),
      chain: s.chain.filter(p => p.id !== id),
    }))
  },
  isInitializing: (uniqueId: string) => {
    return Boolean(get().initializingMap[uniqueId])
  },
}))

listen('chain_updated', () => {
  useChainStore.getState().refresh()
}).catch(() => {})

useChainStore.getState().loadPlaceholders()
