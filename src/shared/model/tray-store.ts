import { create } from 'zustand'
import { createJSONStorage, persist } from 'zustand/middleware'
import { tauriStorage } from '@/shared/lib/tauri-storage'

interface TraySettingsState {
  autostart: boolean
  autostartToTray: boolean
  minimizeToTray: boolean
  setAutostart: (v: boolean) => void
  setAutostartToTray: (v: boolean) => void
  setMinimizeToTray: (v: boolean) => void
}

export const useTrayStore = create<TraySettingsState>()(
  persist(
    set => ({
      autostart: true,
      autostartToTray: true,
      minimizeToTray: true,
      setAutostart: v => set({ autostart: v }),
      setAutostartToTray: v => set({ autostartToTray: v }),
      setMinimizeToTray: v => set({ minimizeToTray: v }),
    }),
    {
      name: 'tray-settings',
      storage: createJSONStorage(() => tauriStorage),
    },
  ),
)
