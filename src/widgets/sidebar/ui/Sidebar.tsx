import type { ReactNode } from 'react'
import { Link, useLocation } from '@tanstack/react-router'
import { AnimatePresence, domAnimation, LazyMotion, m } from 'framer-motion'
import { Mic, Plug, Settings } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { cn } from '@/shared/lib/utils'
import { buttonVariants } from '@/shared/ui/button-variants'

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

  return (
    <Link
      to={to}
      aria-label={label}
      className={cn(
        buttonVariants({ variant: 'ghost', size: 'default' }),
        'h-8 w-full justify-start gap-0 pl-2 pr-0 hover:border-border',
        isActive
          ? 'border-border bg-muted text-foreground'
          : 'text-muted-foreground',
      )}
    >
      {icon}
      <AnimatePresence>
        {!collapsed && (
          <m.span
            initial={{ opacity: 0, x: -10, width: 0 }}
            animate={{ opacity: 1, x: 0, width: 'auto' }}
            exit={{ opacity: 0, x: -10, width: 0 }}
            transition={{ duration: 0.2, ease: 'easeInOut' }}
            className="ml-2 truncate text-sm"
          >
            {label}
          </m.span>
        )}
      </AnimatePresence>
    </Link>
  )
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
