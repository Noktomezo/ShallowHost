import {
  createHashHistory,
  createRootRoute,
  createRoute,
  createRouter,
  Outlet,
  RouterProvider,
  useLocation,
} from '@tanstack/react-router'
import { useEffect } from 'react'
import { HomePage } from '@/pages/home'
import { PluginsPage } from '@/pages/plugins'
import { SettingsPage } from '@/pages/settings'
import { updateService } from '@/shared/lib/updater'
import { useChainStore } from '@/shared/model/chain-store'
import { useUIStore } from '@/shared/model/ui-store'
import { useUpdateStore } from '@/shared/model/update-store'
import { ScrollArea } from '@/shared/ui/scroll-area'
import { isUpdateToastVisible, showUpdateToast } from '@/shared/ui/sonner'
import { TooltipProvider } from '@/shared/ui/tooltip'
import { Sidebar } from '@/widgets/sidebar'
import { Titlebar } from '@/widgets/titlebar'

import './styles.css'

function RootLayout() {
  const collapsed = useUIStore(s => s.sidebarCollapsed)
  const toggle = useUIStore(s => s.toggleSidebar)
  const loading = useChainStore(s => s.loading)
  const loadingMessage = useChainStore(s => s.loadingMessage)
  const location = useLocation()

  return (
    <TooltipProvider>
      <div className="flex h-screen w-screen flex-col overflow-hidden bg-background text-foreground">
        <Titlebar collapsed={collapsed} onToggleCollapse={toggle} />
        <div className="flex min-h-0 flex-1 bg-sidebar">
          <Sidebar collapsed={collapsed} />
          <main className="relative min-w-0 flex-1 overflow-hidden rounded-tl-[8px] bg-background">
            <ScrollArea className="h-full">
              <div className="flex min-h-full flex-col p-4">
                <div key={location.pathname} className="animate-in fade-in slide-in-from-bottom-2 duration-200 ease-out">
                  <Outlet />
                </div>
              </div>
            </ScrollArea>
            {loading && (
              <div className="absolute inset-0 z-50 flex flex-col items-center justify-center bg-background/60 backdrop-blur-md transition-all duration-300">
                <div className="flex flex-col items-center gap-4 rounded-xl border border-border bg-card p-6 shadow-2xl animate-in fade-in zoom-in-95 duration-200">
                  <div className="relative flex h-12 w-12 items-center justify-center">
                    <span className="absolute inline-flex h-full w-full animate-ping rounded-full bg-primary/20 opacity-75"></span>
                    <div className="h-8 w-8 animate-spin rounded-full border-4 border-primary border-t-transparent"></div>
                  </div>
                  {loadingMessage && (
                    <p className="text-sm font-medium text-foreground tracking-wide animate-pulse">
                      {loadingMessage}
                    </p>
                  )}
                </div>
              </div>
            )}
          </main>
        </div>
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
  // ponytail: poll for updates every 30s; re-show after user dismisses (mock always finds one).
  // State shared with SettingsPage via useUpdateStore — manual button disabled while auto-check runs.
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
        // ponytail: network/endpoint errors → treat as up-to-date so UI doesn't
        // stick on 'checking' forever. Logged for diagnosis.
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
