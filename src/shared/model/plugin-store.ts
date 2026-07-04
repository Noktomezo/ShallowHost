import { create } from 'zustand'
import { createJSONStorage, persist } from 'zustand/middleware'
import { tauriStorage } from '@/shared/lib/tauri-storage'

export interface ScannedPlugin {
  name: string
  vendor: string
  version: number
  category: string
  path: string
  unique_id: string
  format: string
  has_editor: boolean
  accepts_midi: boolean
}

interface PluginStore {
  plugins: ScannedPlugin[]
  setPlugins: (p: ScannedPlugin[]) => void
  removePlugin: (id: string) => void
}

export const usePluginStore = create<PluginStore>()(
  persist(
    set => ({
      plugins: [],
      setPlugins: plugins => set({ plugins }),
      removePlugin: id =>
        set(s => ({ plugins: s.plugins.filter(p => p.unique_id !== id) })),
    }),
    {
      name: 'plugins',
      storage: createJSONStorage(() => tauriStorage),
    },
  ),
)
