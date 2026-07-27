import type { ScannedPlugin } from '@/shared/model/plugin-store'
import { Link } from '@tanstack/react-router'
import { ArrowRight, FolderOpen, Loader2, Plus, Trash2 } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { cn } from '@/shared/lib/utils'
import { Badge } from '@/shared/ui/badge'
import { Button } from '@/shared/ui/button'
import { buttonVariants } from '@/shared/ui/button-variants'
import { Card, CardAction, CardDescription, CardHeader, CardTitle } from '@/shared/ui/card'
import { Tooltip, TooltipContent, TooltipTrigger } from '@/shared/ui/tooltip'

interface PluginItemCardProps {
  plugin: ScannedPlugin
  inChain: boolean
  isInitializing: boolean
  onAdd: (p: ScannedPlugin) => void
  onReveal: (path: string) => void
  onRemove: (id: string) => void
}

export function PluginItemCard({
  plugin: p,
  inChain,
  isInitializing,
  onAdd,
  onReveal,
  onRemove,
}: PluginItemCardProps) {
  const { t } = useTranslation()

  return (
    <Card size="sm">
      <CardHeader className="gap-0.5">
        <div className="flex items-center gap-2">
          <CardTitle>{p.name}</CardTitle>
          <Badge variant="purple" className="shrink-0">
            {p.format.toUpperCase()}
          </Badge>
          {isInitializing
            ? (
                <Badge variant="outline" className="gap-1.5 shrink-0 border-amber-500/40 text-amber-400 bg-amber-500/10">
                  <Loader2 className="size-3 animate-spin text-amber-400" />
                  {t('plugins.initializing')}
                </Badge>
              )
            : inChain
              ? (
                  <Badge variant="green" className="shrink-0">
                    {t('plugins.inChain')}
                  </Badge>
                )
              : null}
        </div>
        {p.vendor && <CardDescription>{p.vendor}</CardDescription>}
        <CardAction className="self-center">
          <div className="flex gap-1">
            {inChain || isInitializing
              ? (
                  <Link
                    to="/"
                    className={cn(buttonVariants({ variant: 'default' }))}
                  >
                    {t('plugins.goToChain')}
                    <ArrowRight className="size-4" data-icon="inline-end" />
                  </Link>
                )
              : (
                  <Tooltip>
                    <TooltipTrigger
                      render={(
                        <Button
                          variant="outline"
                          size="icon"
                          aria-label={t('plugins.addToChain')}
                          onClick={() => onAdd(p)}
                        >
                          <Plus className="size-4" />
                        </Button>
                      )}
                    />
                    <TooltipContent>{t('plugins.addToChain')}</TooltipContent>
                  </Tooltip>
                )}
            <Tooltip>
              <TooltipTrigger
                render={(
                  <Button
                    variant="outline"
                    size="icon"
                    aria-label={t('plugins.reveal')}
                    onClick={() => onReveal(p.path)}
                  >
                    <FolderOpen className="size-4" />
                  </Button>
                )}
              />
              <TooltipContent>{t('plugins.reveal')}</TooltipContent>
            </Tooltip>
            <Tooltip>
              <TooltipTrigger
                render={(
                  <Button
                    variant="outline"
                    size="icon"
                    aria-label={t('plugins.remove')}
                    className="hover:bg-destructive/15 hover:text-destructive"
                    onClick={() => onRemove(p.unique_id)}
                  >
                    <Trash2 className="size-4" />
                  </Button>
                )}
              />
              <TooltipContent>{t('plugins.remove')}</TooltipContent>
            </Tooltip>
          </div>
        </CardAction>
      </CardHeader>
    </Card>
  )
}
