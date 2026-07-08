import type { UpdateInfo } from '@/shared/lib/updater'
import { create } from 'zustand'
import { createJSONStorage, persist } from 'zustand/middleware'
import { tauriStorage } from '@/shared/lib/tauri-storage'

type CheckResult
  = | { kind: 'idle' }
    | { kind: 'checking' }
    | { kind: 'up-to-date' }
    | { kind: 'available', info: UpdateInfo }

interface UpdateStoreState {
  checkResult: CheckResult
  autoCheckEnabled: boolean
  setCheckResult: (r: CheckResult) => void
  setAutoCheckEnabled: (v: boolean) => void
}

export const useUpdateStore = create<UpdateStoreState>()(
  persist(
    set => ({
      checkResult: { kind: 'idle' },
      autoCheckEnabled: true,
      setCheckResult: checkResult => set({ checkResult }),
      setAutoCheckEnabled: autoCheckEnabled => set({ autoCheckEnabled }),
    }),
    {
      name: 'update-settings',
      storage: createJSONStorage(() => tauriStorage),
      // ponytail: checkResult is transient — only persist the toggle.
      partialize: state => ({ autoCheckEnabled: state.autoCheckEnabled }),
    },
  ),
)
