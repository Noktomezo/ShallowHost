import { Button as ButtonPrimitive } from '@base-ui/react/button'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { PanelLeftClose, PanelLeftOpen } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/shared/ui/button'

const appWindow = getCurrentWindow()

interface TitlebarProps {
  collapsed: boolean
  onToggleCollapse: () => void
}

function IconClose() {
  return (
    <svg className="size-2 stroke-black/70 dark:stroke-white/70 stroke-[1.5]" viewBox="0 0 6 6">
      <line x1="1" y1="1" x2="5" y2="5" />
      <line x1="5" y1="1" x2="1" y2="5" />
    </svg>
  )
}

function IconMinimize() {
  return (
    <svg className="size-2 stroke-black/70 dark:stroke-white/70 stroke-[1.5]" viewBox="0 0 6 6">
      <line x1="1" y1="3" x2="5" y2="3" />
    </svg>
  )
}

function IconMaximize() {
  return (
    <svg className="size-2 stroke-black/70 dark:stroke-white/70 stroke-[1.2]" fill="none" viewBox="0 0 6 6">
      <path d="M1.5 4.5 L4.5 1.5 M4.5 3.5 V1.5 H2.5 M1.5 2.5 V4.5 H3.5" />
    </svg>
  )
}

export function Titlebar({ collapsed, onToggleCollapse }: TitlebarProps) {
  const { t } = useTranslation()
  return (
    <header
      data-tauri-drag-region
      className="flex shrink-0 items-center justify-between bg-sidebar select-none py-1.5"
    >
      <div className="flex h-full items-center gap-1 pl-1">
        <Button
          variant="ghost"
          size="icon"
          onClick={onToggleCollapse}
          aria-label={t('titlebar.toggleSidebar')}
          className="hover:border-border"
        >
          {collapsed
            ? (
                <PanelLeftOpen className="size-4" />
              )
            : (
                <PanelLeftClose className="size-4" />
              )}
        </Button>
      </div>

      {/* macOS Traffic Lights Style Window Controls */}
      <div className="group/traffic flex items-center gap-2 pr-4 pl-2 h-8">
        {/* Close (Red) */}
        <ButtonPrimitive
          onClick={() => appWindow.close()}
          aria-label={t('titlebar.close')}
          className="group/btn relative flex items-center justify-center size-3.5 rounded-full bg-ring transition-all duration-200 ease-out group-hover/traffic:bg-red cursor-pointer"
        >
          <span className="opacity-0 group-hover/btn:opacity-100 transition-opacity duration-150 absolute inset-0 flex items-center justify-center">
            <IconClose />
          </span>
        </ButtonPrimitive>

        {/* Minimize (Yellow) */}
        <ButtonPrimitive
          onClick={() => appWindow.minimize()}
          aria-label={t('titlebar.minimize')}
          className="group/btn relative flex items-center justify-center size-3.5 rounded-full bg-ring transition-all duration-200 ease-out group-hover/traffic:bg-yellow cursor-pointer"
        >
          <span className="opacity-0 group-hover/btn:opacity-100 transition-opacity duration-150 absolute inset-0 flex items-center justify-center">
            <IconMinimize />
          </span>
        </ButtonPrimitive>

        {/* Maximize (Green) */}
        <ButtonPrimitive
          onClick={() => appWindow.toggleMaximize()}
          aria-label={t('titlebar.maximize')}
          className="group/btn relative flex items-center justify-center size-3.5 rounded-full bg-ring transition-all duration-200 ease-out group-hover/traffic:bg-green cursor-pointer"
        >
          <span className="opacity-0 group-hover/btn:opacity-100 transition-opacity duration-150 absolute inset-0 flex items-center justify-center">
            <IconMaximize />
          </span>
        </ButtonPrimitive>
      </div>
    </header>
  )
}
