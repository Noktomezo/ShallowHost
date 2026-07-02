import {
  createHashHistory,
  createRootRoute,
  createRoute,
  createRouter,
  Outlet,
  RouterProvider,
} from "@tanstack/react-router";
import { Titlebar } from "@/widgets/titlebar";
import { Sidebar } from "@/widgets/sidebar";
import { ScrollArea } from "@/shared/ui/scroll-area";
import { TooltipProvider } from "@/shared/ui/tooltip";
import { useUIStore } from "@/shared/model/ui-store";
import { HomePage } from "@/pages/home";
import { PluginsPage } from "@/pages/plugins";
import { SettingsPage } from "@/pages/settings";
import "./styles.css";

function RootLayout() {
  const collapsed = useUIStore((s) => s.sidebarCollapsed);
  const toggle = useUIStore((s) => s.toggleSidebar);

  return (
    <TooltipProvider>
      <div className="flex h-screen w-screen flex-col overflow-hidden bg-background text-foreground">
        <Titlebar collapsed={collapsed} onToggleCollapse={toggle} />
        <div className="flex min-h-0 flex-1">
          <Sidebar collapsed={collapsed} />
          <main className="min-w-0 flex-1 overflow-hidden rounded-tl-lg bg-background">
            <ScrollArea className="h-full">
              <div className="flex min-h-full flex-col p-4">
                <Outlet />
              </div>
            </ScrollArea>
          </main>
        </div>
      </div>
    </TooltipProvider>
  );
}

const rootRoute = createRootRoute({ component: RootLayout });
const homeRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/",
  component: HomePage,
});
const pluginsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/plugins",
  component: PluginsPage,
});
const settingsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/settings",
  component: SettingsPage,
});

const routeTree = rootRoute.addChildren([homeRoute, pluginsRoute, settingsRoute]);

export const router = createRouter({
  routeTree,
  history: createHashHistory(),
});

declare module "@tanstack/react-router" {
  interface Register {
    router: typeof router;
  }
}

export function App() {
  return <RouterProvider router={router} />;
}
