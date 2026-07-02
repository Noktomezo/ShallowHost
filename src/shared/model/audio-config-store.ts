import { create } from "zustand";
import { persist, createJSONStorage } from "zustand/middleware";
import { invoke } from "@tauri-apps/api/core";
import { tauriStorage } from "@/shared/lib/tauri-storage";

export interface AudioConfig {
  driver: string;
  input_device: string | null;
  output_device: string | null;
  sample_rate: number;
  buffer_size: number;
  mono: boolean;
}

interface AudioConfigState {
  config: AudioConfig;
  setConfig: (config: AudioConfig) => void;
  updateConfig: (patch: Partial<AudioConfig>) => void;
  loadFromBackend: () => Promise<void>;
}

const DEFAULT_CONFIG: AudioConfig = {
  driver: "wasapi",
  input_device: null,
  output_device: null,
  sample_rate: 48000,
  buffer_size: 512,
  mono: false,
};

export const useAudioConfigStore = create<AudioConfigState>()(
  persist(
    (set, get) => ({
      config: DEFAULT_CONFIG,
      setConfig: (config) => set({ config }),
      updateConfig: (patch) => {
        const next = { ...get().config, ...patch };
        set({ config: next });
      },
      loadFromBackend: async () => {
        try {
          const result = await invoke<AudioConfig>("get_audio_config");
          set({ config: result });
        } catch {
          // backend not ready yet
        }
      },
    }),
    {
      name: "audio-config",
      storage: createJSONStorage(() => tauriStorage),
    },
  ),
);
