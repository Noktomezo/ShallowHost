import React, { createContext, useContext, useEffect, useMemo } from 'react'
import { useThemeStore } from '@/shared/model/theme-store'

interface ThemeProviderProps {
  children: React.ReactNode
  defaultTheme?: 'system' | 'dark' | 'light'
}

interface ThemeContextType {
  theme: 'system' | 'dark' | 'light'
  setTheme: (theme: 'system' | 'dark' | 'light') => void
}

const ThemeContext = createContext<ThemeContextType>({
  theme: 'system',
  setTheme: () => {},
})

export function ThemeProvider({ children, defaultTheme }: ThemeProviderProps) {
  const { theme, setTheme } = useThemeStore()

  useEffect(() => {
    if (!defaultTheme)
      return

    if (useThemeStore.persist.hasHydrated()) {
      const currentTheme = useThemeStore.getState().theme
      if (!currentTheme) {
        setTheme(defaultTheme)
      }
    }
    else {
      const unsub = useThemeStore.persist.onFinishHydration((state) => {
        if (!state?.theme) {
          setTheme(defaultTheme)
        }
      })
      return () => unsub()
    }
  }, [defaultTheme, setTheme])

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      const target = e.target as HTMLElement
      if (
        target.tagName === 'INPUT'
        || target.tagName === 'TEXTAREA'
        || target.tagName === 'SELECT'
        || target.isContentEditable
      ) {
        return
      }

      if (e.key.toLowerCase() === 'd') {
        e.preventDefault()
        setTheme(theme === 'dark' ? 'light' : 'dark')
      }
    }
    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [theme, setTheme])

  const contextValue = useMemo(() => ({ theme, setTheme }), [theme, setTheme])

  return (
    <ThemeContext.Provider value={contextValue}>
      {children}
    </ThemeContext.Provider>
  )
}

export const useTheme = () => useContext(ThemeContext)
