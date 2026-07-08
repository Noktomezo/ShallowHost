import { invoke } from '@tauri-apps/api/core'
import { create } from 'zustand'
import { createJSONStorage, persist } from 'zustand/middleware'
import { tauriStorage } from '@/shared/lib/tauri-storage'

export interface AudioConfig {
  driver: string
  input_device: string | null
  output_device: string | null
  sample_rate: number
  buffer_size: number
  mono: boolean
  active_inputs?: number[] | null
  active_outputs?: number[] | null
}

interface AudioConfigState {
  config: AudioConfig
  lastAsioDevice: string | null
  lastAsioInputs: number[] | null
  lastAsioOutputs: number[] | null
  lastWasapiInput: string | null
  lastWasapiOutput: string | null
  setConfig: (config: AudioConfig) => void
  updateConfig: (patch: Partial<AudioConfig>) => void
  loadFromBackend: () => Promise<void>
}

const DEFAULT_CONFIG: AudioConfig = {
  driver: 'wasapi',
  input_device: null,
  output_device: null,
  sample_rate: 48000,
  buffer_size: 512,
  mono: false,
  active_inputs: null,
  active_outputs: null,
}

export const useAudioConfigStore = create<AudioConfigState>()(
  persist(
    (set, get) => ({
      config: DEFAULT_CONFIG,
      lastAsioDevice: null,
      lastAsioInputs: null,
      lastAsioOutputs: null,
      lastWasapiInput: null,
      lastWasapiOutput: null,
      setConfig: config => set({ config }),
      updateConfig: (patch) => {
        const next = { ...get().config, ...patch }
        set({ config: next })
      },
      loadFromBackend: async () => {
        try {
          const result = await invoke<AudioConfig>('get_audio_config')
          set({ config: result })
        }
        catch {
          // backend not ready yet
        }
      },
    }),
    {
      name: 'audio-config',
      storage: createJSONStorage(() => tauriStorage),
    },
  ),
)
