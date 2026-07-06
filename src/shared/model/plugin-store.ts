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

const DEFAULT_VST2_PATHS = [
  'C:\\Program Files\\VSTPlugins',
  'C:\\Program Files\\Common Files\\VST2',
  'C:\\Program Files (x86)\\VSTPlugins',
]

const DEFAULT_VST3_PATHS = [
  'C:\\Program Files\\Common Files\\VST3',
  'C:\\Program Files (x86)\\Common Files\\VST3',
]

interface PluginStore {
  plugins: ScannedPlugin[]
  vst2Paths: string[]
  vst3Paths: string[]
  setPlugins: (p: ScannedPlugin[]) => void
  removePlugin: (id: string) => void
  addVst2Path: (path: string) => void
  removeVst2Path: (path: string) => void
  addVst3Path: (path: string) => void
  removeVst3Path: (path: string) => void
  resetPaths: () => void
}

export const usePluginStore = create<PluginStore>()(
  persist(
    set => ({
      plugins: [],
      vst2Paths: DEFAULT_VST2_PATHS,
      vst3Paths: DEFAULT_VST3_PATHS,
      setPlugins: plugins => set({ plugins }),
      removePlugin: id =>
        set(s => ({ plugins: s.plugins.filter(p => p.unique_id !== id) })),
      addVst2Path: path =>
        set(s => s.vst2Paths.includes(path) ? {} : { vst2Paths: [...s.vst2Paths, path] }),
      removeVst2Path: path =>
        set(s => ({ vst2Paths: s.vst2Paths.filter(p => p !== path) })),
      addVst3Path: path =>
        set(s => s.vst3Paths.includes(path) ? {} : { vst3Paths: [...s.vst3Paths, path] }),
      removeVst3Path: path =>
        set(s => ({ vst3Paths: s.vst3Paths.filter(p => p !== path) })),
      resetPaths: () =>
        set({ vst2Paths: DEFAULT_VST2_PATHS, vst3Paths: DEFAULT_VST3_PATHS }),
    }),
    {
      name: 'plugins',
      storage: createJSONStorage(() => tauriStorage),
    },
  ),
)
