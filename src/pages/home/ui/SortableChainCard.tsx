import type { ChainItem } from '@/shared/model/chain-store'
import { useSortable } from '@dnd-kit/sortable'
import { CSS } from '@dnd-kit/utilities'
import { GripVertical } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Badge } from '@/shared/ui/badge'
import { Button } from '@/shared/ui/button'
import {
  Card,
  CardAction,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/shared/ui/card'
import { ChainCardActions } from './ChainCardActions'

export function SortableChainCard({ plugin: p }: { plugin: ChainItem }) {
  const { t } = useTranslation()
  const { attributes, listeners, setNodeRef, transform, transition, isDragging } = useSortable({
    id: p.id,
  })

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
    opacity: isDragging ? 0.5 : 1,
  }

  return (
    <Card ref={setNodeRef} size="sm" style={style}>
      <CardHeader className="gap-0.5">
        <div className="flex items-center gap-3">
          <Button
            variant="outline"
            size="icon"
            className="cursor-grab active:cursor-grabbing touch-none"
            aria-label={t('home.dragHandle')}
            {...attributes}
            {...listeners}
          >
            <GripVertical className="size-4 text-muted-foreground" />
          </Button>
          <div className="flex flex-col gap-0">
            <div className="flex items-center gap-2">
              <CardTitle className={p.bypassed ? 'text-muted-foreground' : ''}>
                {p.name}
              </CardTitle>
              <Badge variant="secondary" className="shrink-0">
                {p.format.toUpperCase()}
              </Badge>
              {p.bypassed && (
                <Badge variant="destructive" className="shrink-0">
                  {t('home.bypassed')}
                </Badge>
              )}
            </div>
            <CardDescription>{p.vendor || ''}</CardDescription>
          </div>
        </div>
        <CardAction className="self-center">
          <ChainCardActions plugin={p} />
        </CardAction>
      </CardHeader>
    </Card>
  )
}
