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

  const toggleTheme = () => {
    const nextTheme = theme === 'dark' ? 'light' : theme === 'light' ? 'system' : 'dark'
    setTheme(nextTheme)
  }

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && (e.key === 'd' || e.key === 't' || e.code === 'KeyD')) {
        e.preventDefault()
        toggleTheme()
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
