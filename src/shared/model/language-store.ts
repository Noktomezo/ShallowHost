import { create } from 'zustand'
import { createJSONStorage, persist } from 'zustand/middleware'
import i18n from '@/shared/config/i18n'
import { tauriStorage } from '@/shared/lib/tauri-storage'

type Language = 'ru' | 'en'

interface LanguageState {
  language: Language
  setLanguage: (lang: Language) => void
}

export const useLanguageStore = create<LanguageState>()(
  persist(
    set => ({
      language: 'ru',
      setLanguage: (language) => {
        i18n.changeLanguage(language)
        set({ language })
      },
    }),
    {
      name: 'language',
      storage: createJSONStorage(() => tauriStorage),
      onRehydrateStorage: () => (state) => {
        if (state)
          i18n.changeLanguage(state.language)
      },
    },
  ),
)

if (typeof window !== 'undefined') {
  i18n.changeLanguage('ru')
}
