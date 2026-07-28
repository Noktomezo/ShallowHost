import { useNavigate } from '@tanstack/react-router'
import { Home, Moon, Package, Settings, Sun, Trash2 } from 'lucide-react'
import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { clearChainWithUndo } from '@/pages/home/lib/clear-chain-action'
import { useThemeStore } from '@/shared/model/theme-store'
import {
  CommandDialog,
  CommandEmpty,
  CommandInput,
  CommandItem,
  CommandList,
} from '@/shared/ui/command'

export function CommandMenu() {
  const [open, setOpen] = useState(false)
  const [search, setSearch] = useState('')
  const { t } = useTranslation()
  const navigate = useNavigate()
  const { theme, setTheme } = useThemeStore()

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'k') {
        e.preventDefault()
        setOpen(o => !o)
      }
    }
    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [])

  const actions = [
    {
      id: 'home',
      icon: Home,
      label: t('sidebar.home'),
      perform: () => {
        navigate({ to: '/' })
        setOpen(false)
      },
    },
    {
      id: 'plugins',
      icon: Package,
      label: t('sidebar.plugins'),
      perform: () => {
        navigate({ to: '/plugins' })
        setOpen(false)
      },
    },
    {
      id: 'settings',
      icon: Settings,
      label: t('sidebar.settings'),
      perform: () => {
        navigate({ to: '/settings' })
        setOpen(false)
      },
    },
    {
      id: 'theme',
      icon: theme === 'dark' ? Sun : Moon,
      label: theme === 'dark' ? t('command.switchToLight') : t('command.switchToDark'),
      perform: () => {
        setTheme(theme === 'dark' ? 'light' : 'dark')
        setOpen(false)
      },
    },
    {
      id: 'clear-chain',
      icon: Trash2,
      label: t('home.clearChain'),
      perform: () => {
        clearChainWithUndo(t)
        setOpen(false)
      },
    },
  ]

  const filtered = actions.filter(a =>
    a.label.toLowerCase().includes(search.toLowerCase()),
  )

  return (
    <CommandDialog open={open} onOpenChange={setOpen}>
      <CommandInput
        value={search}
        onValueChange={setSearch}
        placeholder={t('command.searchPlaceholder')}
      />
      <CommandList>
        {filtered.length === 0
          ? (
              <CommandEmpty>{t('command.noResults')}</CommandEmpty>
            )
          : (
              filtered.map((action) => {
                const Icon = action.icon
                return (
                  <CommandItem key={action.id} onSelect={action.perform}>
                    <Icon className="size-4 shrink-0 text-muted-foreground" />
                    <span>{action.label}</span>
                  </CommandItem>
                )
              })
            )}
      </CommandList>
    </CommandDialog>
  )
}
