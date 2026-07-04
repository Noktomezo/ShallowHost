import { create } from 'zustand'
import { createJSONStorage, persist } from 'zustand/middleware'
import { tauriStorage } from '@/shared/lib/tauri-storage'

interface UIState {
  sidebarCollapsed: boolean
  toggleSidebar: () => void
}

export const useUIStore = create<UIState>()(
  persist(
    set => ({
      sidebarCollapsed: false,
      toggleSidebar: () => set(s => ({ sidebarCollapsed: !s.sidebarCollapsed })),
    }),
    {
      name: 'ui',
      storage: createJSONStorage(() => tauriStorage),
    },
  ),
)
