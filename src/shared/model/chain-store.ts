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
  updateChainItem: (id: string, patch: Partial<ChainItem>) => void
  refresh: () => Promise<void>
  loadPlaceholders: () => Promise<void>
  addPluginAsync: (plugin: { unique_id: string, name: string, vendor: string, format: string }) => Promise<ChainItem | undefined>
  remove: (id: string) => void
  isInitializing: (uniqueId: string) => boolean
}

function computeChain(rawChain: ChainItem[], initializingMap: Record<string, ChainItem>): ChainItem[] {
  const pending = Object.values(initializingMap).filter(
    item => !rawChain.some(c => c.unique_id === item.unique_id),
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
  updateChainItem: (id, patch) =>
    set((s) => {
      const nextRaw = s.rawChain.map(p => (p.id === id ? { ...p, ...patch } : p))
      return {
        rawChain: nextRaw,
        chain: computeChain(nextRaw, s.initializingMap),
      }
    }),
  loadPlaceholders: async () => {
    try {
      const placeholders = await invoke<ChainItem[]>('get_saved_chain_placeholders')
      if (placeholders.length > 0) {
        const nextMap: Record<string, ChainItem> = {}
        for (const item of placeholders) {
          if (item.unique_id) {
            nextMap[item.unique_id] = { ...item, initializing: true }
            const uid = item.unique_id
            setTimeout(() => {
              set((s) => {
                if (s.initializingMap[uid]) {
                  const map = { ...s.initializingMap }
                  delete map[uid]
                  return {
                    initializingMap: map,
                    chain: computeChain(s.rawChain, map),
                  }
                }
                return s
              })
            }, 15000)
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
  addPluginAsync: async (plugin) => {
    if (get().initializingMap[plugin.unique_id]) {
      return undefined
    }

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

    let timer: ReturnType<typeof setTimeout>
    const timeoutPromise = new Promise((_, reject) => {
      timer = setTimeout(() => {
        reject(new Error(`Loading ${plugin.name} timed out`))
      }, 15000)
    })

    try {
      await Promise.race([
        invoke('add_to_chain', { pluginId: plugin.unique_id }),
        timeoutPromise,
      ])
      clearTimeout(timer!)
      await get().refresh()
      return get().rawChain.find(p => p.unique_id === plugin.unique_id)
    }
    catch (e) {
      clearTimeout(timer!)
      set((s) => {
        const nextMap = { ...s.initializingMap }
        delete nextMap[plugin.unique_id]
        return {
          initializingMap: nextMap,
          chain: computeChain(s.rawChain, nextMap),
          error: String(e),
        }
      })
      throw e
    }
    finally {
      if (get().initializingMap[plugin.unique_id]) {
        set((s) => {
          const nextMap = { ...s.initializingMap }
          delete nextMap[plugin.unique_id]
          return {
            initializingMap: nextMap,
            chain: computeChain(s.rawChain, nextMap),
          }
        })
      }
    }
  },
  remove: (id) => {
    set((s) => {
      const nextMap = { ...s.initializingMap }
      const item = s.rawChain.find(p => p.id === id) || s.chain.find(p => p.id === id)
      if (item?.unique_id && nextMap[item.unique_id]) {
        delete nextMap[item.unique_id]
      }
      if (id.startsWith('temp-')) {
        const uid = id.replace('temp-', '')
        delete nextMap[uid]
      }
      const nextRaw = s.rawChain.filter(p => p.id !== id)
      return {
        rawChain: nextRaw,
        initializingMap: nextMap,
        chain: computeChain(nextRaw, nextMap),
      }
    })
  },
  isInitializing: (uniqueId: string) => {
    return Boolean(get().initializingMap[uniqueId])
  },
}))

let unlistenChainUpdated: (() => void) | undefined

if (unlistenChainUpdated) {
  unlistenChainUpdated()
  unlistenChainUpdated = undefined
}

listen('chain_updated', () => {
  useChainStore.getState().refresh()
})
  .then((unlisten) => {
    unlistenChainUpdated = unlisten
  })
  .catch(() => {})

useChainStore.getState().loadPlaceholders()
