import {
  createHashHistory,
  createRootRoute,
  createRoute,
  createRouter,
  Link,
  Outlet,
  RouterProvider,
  useLocation,
} from '@tanstack/react-router'
import { Menu, Mic, Plug, Settings, X } from 'lucide-react'
import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { HomePage } from '@/pages/home'
import { PluginsPage } from '@/pages/plugins'
import { SettingsPage } from '@/pages/settings'
import { updateService } from '@/shared/lib/updater'
import { useUIStore } from '@/shared/model/ui-store'
import { useUpdateStore } from '@/shared/model/update-store'
import { ScrollArea } from '@/shared/ui/scroll-area'
import { isUpdateToastVisible, showUpdateToast } from '@/shared/ui/sonner-utils'
import { TooltipProvider } from '@/shared/ui/tooltip'
import { CommandMenu } from '@/widgets/command-menu/ui/CommandMenu'
import { Sidebar } from '@/widgets/sidebar'
import { Titlebar } from '@/widgets/titlebar'

import './styles.css'

function RootLayout() {
  const collapsed = useUIStore(s => s.sidebarCollapsed)
  const toggle = useUIStore(s => s.toggleSidebar)
  const location = useLocation()
  const { t } = useTranslation()
  const [mobileNavOpen, setMobileNavOpen] = useState(false)

  return (
    <TooltipProvider>
      <div className="flex h-screen w-screen flex-col overflow-hidden bg-background text-foreground">
        <Titlebar collapsed={collapsed} onToggleCollapse={toggle} />
        <div className="flex min-h-0 flex-1 bg-sidebar">
          <Sidebar collapsed={collapsed} />
          <main className="relative min-w-0 flex-1 overflow-hidden rounded-tl-[8px] bg-background">
            <div className="md:hidden flex items-center justify-between border-b border-border p-2 bg-sidebar">
              <button
                type="button"
                aria-label={t('titlebar.toggleMobileNav')}
                aria-expanded={mobileNavOpen}
                aria-controls="mobile-navigation-panel"
                className="flex items-center gap-2 rounded-md px-3 py-1.5 text-xs font-medium bg-muted hover:bg-muted/80 cursor-pointer"
                onClick={() => setMobileNavOpen(!mobileNavOpen)}
              >
                {mobileNavOpen ? <X className="size-4" /> : <Menu className="size-4" />}
                <span>{t('titlebar.navigation')}</span>
              </button>
            </div>
            {mobileNavOpen && (
              <div
                id="mobile-navigation-panel"
                className="md:hidden absolute inset-0 z-50 bg-background/95 backdrop-blur-sm p-4 flex flex-col gap-4 animate-in fade-in"
              >
                <nav aria-label={t('titlebar.mobileNav')} className="flex flex-col gap-2">
                  <Link
                    to="/"
                    onClick={() => setMobileNavOpen(false)}
                    className="flex items-center gap-2 p-2 rounded-md hover:bg-accent text-sm font-medium"
                  >
                    <Mic className="size-4" />
                    <span>{t('sidebar.home')}</span>
                  </Link>
                  <Link
                    to="/plugins"
                    onClick={() => setMobileNavOpen(false)}
                    className="flex items-center gap-2 p-2 rounded-md hover:bg-accent text-sm font-medium"
                  >
                    <Plug className="size-4" />
                    <span>{t('sidebar.plugins')}</span>
                  </Link>
                  <Link
                    to="/settings"
                    onClick={() => setMobileNavOpen(false)}
                    className="flex items-center gap-2 p-2 rounded-md hover:bg-accent text-sm font-medium"
                  >
                    <Settings className="size-4" />
                    <span>{t('sidebar.settings')}</span>
                  </Link>
                </nav>
              </div>
            )}
            <ScrollArea key={location.pathname} className="h-full">
              <div className="flex min-h-full flex-col p-4">
                <div key={location.pathname} className="transition-opacity transition-transform duration-200 ease-out animate-in fade-in slide-in-from-bottom-2">
                  <Outlet />
                </div>
              </div>
            </ScrollArea>
          </main>
        </div>
        <CommandMenu />
      </div>
    </TooltipProvider>
  )
}

const rootRoute = createRootRoute({ component: RootLayout })
const homeRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/',
  component: HomePage,
})
const pluginsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/plugins',
  component: PluginsPage,
})
const settingsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/settings',
  component: SettingsPage,
})

const routeTree = rootRoute.addChildren([homeRoute, pluginsRoute, settingsRoute])

const router = createRouter({
  routeTree,
  history: createHashHistory(),
  defaultPreload: 'intent',
})

declare module '@tanstack/react-router' {
  interface Register {
    router: typeof router
  }
}

export function App() {
  useEffect(() => {
    let cancelled = false
    const run = async () => {
      const { autoCheckEnabled, checkResult, setCheckResult } = useUpdateStore.getState()
      if (!autoCheckEnabled)
        return
      if (checkResult.kind === 'checking')
        return
      if (isUpdateToastVisible())
        return
      setCheckResult({ kind: 'checking' })
      try {
        const info = await updateService.check()
        if (cancelled)
          return
        if (info) {
          setCheckResult({ kind: 'available', info })
          if (!isUpdateToastVisible())
            showUpdateToast(info)
        }
        else {
          setCheckResult({ kind: 'up-to-date' })
        }
      }
      catch (e) {
        if (cancelled)
          return
        console.error('[update] auto-check failed:', e)
        setCheckResult({ kind: 'up-to-date' })
      }
    }
    const initial = setTimeout(run, 5000)
    const interval = setInterval(run, 30000)
    return () => {
      cancelled = true
      clearTimeout(initial)
      clearInterval(interval)
    }
  }, [])
  return <RouterProvider router={router} />
}
