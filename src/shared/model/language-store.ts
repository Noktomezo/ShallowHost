import { create } from 'zustand'
import { createJSONStorage, persist } from 'zustand/middleware'
import i18n from '@/shared/config/i18n'
import { tauriStorage } from '@/shared/lib/tauri-storage'

type Language = 'system' | 'ru' | 'en'

interface LanguageState {
  language: Language
  setLanguage: (lang: Language) => void
}

function getSystemLanguage(): 'ru' | 'en' {
  if (typeof navigator !== 'undefined') {
    const lang = (navigator.language || navigator.languages?.[0] || 'en').toLowerCase()
    if (lang.startsWith('ru'))
      return 'ru'
  }
  return 'en'
}

export const useLanguageStore = create<LanguageState>()(
  persist(
    set => ({
      language: 'system',
      setLanguage: (language) => {
        const resolved = language === 'system' ? getSystemLanguage() : language
        i18n.changeLanguage(resolved)
        set({ language })
      },
    }),
    {
      name: 'language',
      storage: createJSONStorage(() => tauriStorage),
      onRehydrateStorage: () => (state) => {
        if (state) {
          const resolved = state.language === 'system' ? getSystemLanguage() : state.language
          i18n.changeLanguage(resolved)
        }
      },
    },
  ),
)
