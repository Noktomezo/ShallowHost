import type { ReactNode } from 'react'
import { Link, useLocation } from '@tanstack/react-router'
import { AnimatePresence, domAnimation, LazyMotion, m } from 'framer-motion'
import { Mic, Plug, Settings } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { cn } from '@/shared/lib/utils'
import { Tooltip, TooltipContent, TooltipTrigger } from '@/shared/ui/tooltip'

interface SidebarProps {
  collapsed: boolean
}

type NavTo = '/' | '/plugins' | '/settings'

// ponytail: icon in fixed 32px span (aligns with collapse button), text slides+fades
function SideNavItem({
  to,
  label,
  icon,
  collapsed,
}: {
  to: NavTo
  label: string
  icon: ReactNode
  collapsed: boolean
}) {
  const location = useLocation()
  const isActive = location.pathname === to

  const link = (
    <Link
      to={to}
      aria-label={label}
      className={cn(
        'group/button inline-flex shrink-0 items-center justify-start rounded-lg text-sm font-medium whitespace-nowrap transition-all outline-none select-none focus-visible:ring-3 focus-visible:ring-ring/50 active:not-aria-[haspopup]:translate-y-px disabled:pointer-events-none disabled:opacity-50 [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*=\'size-\'])]:size-4',
        'h-8 w-full gap-0 pl-2 pr-0',
        isActive
          ? 'bg-primary text-primary-foreground hover:bg-primary-hover'
          : 'text-muted-foreground hover:bg-primary/10 hover:text-primary',
      )}
    >
      {icon}
      <AnimatePresence>
        {!collapsed && (
          <m.span
            initial={{ opacity: 0, x: -10 }}
            animate={{ opacity: 1, x: 0 }}
            exit={{ opacity: 0, x: -10 }}
            transition={{ duration: 0.2, ease: 'easeInOut' }}
            className="ml-2 truncate text-sm"
          >
            {label}
          </m.span>
        )}
      </AnimatePresence>
    </Link>
  )

  if (collapsed) {
    return (
      <Tooltip>
        <TooltipTrigger render={link} />
        <TooltipContent side="right">{label}</TooltipContent>
      </Tooltip>
    )
  }

  return link
}

export function Sidebar({ collapsed }: SidebarProps) {
  const { t } = useTranslation()
  return (
    <LazyMotion strict features={domAnimation}>
      <nav
        className={cn(
          'flex shrink-0 flex-col items-start overflow-hidden bg-sidebar py-1 px-1 gap-1 transition-[width] duration-200 ease-in-out',
          collapsed ? 'w-10' : 'w-36',
        )}
      >
        <SideNavItem
          to="/"
          label={t('sidebar.home')}
          icon={<Mic className="size-4" />}
          collapsed={collapsed}
        />
        <SideNavItem
          to="/plugins"
          label={t('sidebar.plugins')}
          icon={<Plug className="size-4" />}
          collapsed={collapsed}
        />
        <div className="mt-auto w-full">
          <SideNavItem
            to="/settings"
            label={t('sidebar.settings')}
            icon={<Settings className="size-4" />}
            collapsed={collapsed}
          />
        </div>
      </nav>
    </LazyMotion>
  )
}
