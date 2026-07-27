import React, { createContext, useContext, useEffect } from 'react'
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

export function ThemeProvider({ children }: ThemeProviderProps) {
  const { theme, setTheme } = useThemeStore()

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

  return (
    <ThemeContext.Provider value={{ theme, setTheme }}>
      {children}
    </ThemeContext.Provider>
  )
}

export const useTheme = () => useContext(ThemeContext)
