import { getCurrentWindow } from '@tauri-apps/api/window'
import { Minus, PanelLeftClose, PanelLeftOpen, Square, X } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/shared/ui/button'

const appWindow = getCurrentWindow()

interface TitlebarProps {
  collapsed: boolean
  onToggleCollapse: () => void
}

export function Titlebar({ collapsed, onToggleCollapse }: TitlebarProps) {
  const { t } = useTranslation()
  return (
    <header
      data-tauri-drag-region
      className="flex shrink-0 items-center justify-between bg-sidebar select-none py-1"
    >
      <div className="flex h-full items-center gap-1 pl-1">
        <Button
          variant="ghost"
          size="icon"
          onClick={onToggleCollapse}
          aria-label={t('titlebar.toggleSidebar')}
          className="hover:border-border cursor-pointer"
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

      {/* Symmetrical square buttons with traffic-light highlights */}
      <div className="group/traffic flex h-full items-center gap-1 pr-1 pl-1">
        {/* Minimize */}
        <Button
          variant="ghost"
          size="icon"
          onClick={() => appWindow.minimize()}
          aria-label={t('titlebar.minimize')}
          className="text-muted-foreground border border-transparent transition-all duration-200 group-hover/traffic:text-yellow hover:!bg-yellow hover:!text-primary-foreground hover:!border-yellow/20 cursor-pointer"
        >
          <Minus className="size-4" />
        </Button>

        {/* Maximize */}
        <Button
          variant="ghost"
          size="icon"
          onClick={() => appWindow.toggleMaximize()}
          aria-label={t('titlebar.maximize')}
          className="text-muted-foreground border border-transparent transition-all duration-200 group-hover/traffic:text-green hover:!bg-green hover:!text-primary-foreground hover:!border-green/20 cursor-pointer"
        >
          <Square className="size-3.5" />
        </Button>

        {/* Close */}
        <Button
          variant="ghost"
          size="icon"
          onClick={() => appWindow.close()}
          aria-label={t('titlebar.close')}
          className="text-muted-foreground border border-transparent transition-all duration-200 group-hover/traffic:text-red hover:!bg-red hover:!text-destructive-foreground hover:!border-red/20 cursor-pointer"
        >
          <X className="size-4" />
        </Button>
      </div>
    </header>
  )
}
