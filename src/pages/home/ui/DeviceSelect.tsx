import type React from 'react'
import { useTranslation } from 'react-i18next'
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/shared/ui/select'

export interface DeviceInfo {
  name: string
  default: boolean
}

export function DeviceSelect({
  label,
  description,
  value,
  items,
  devices,
  onChange,
  defaultLabel,
  hideDefault = false,
  meter,
}: {
  label: string
  description: string
  value: string
  items: Record<string, React.ReactNode>
  devices: DeviceInfo[]
  onChange: (v: string | null) => void
  defaultLabel: string
  hideDefault?: boolean
  meter?: React.ReactNode
}) {
  const { t } = useTranslation()

  return (
    <div className="flex items-center justify-between gap-2">
      <div className="flex flex-col gap-0">
        <span className="text-sm font-medium">{label}</span>
        <span className="text-xs text-muted-foreground">{description}</span>
      </div>
      <div className="flex items-center gap-3">
        {meter}
        <Select
          value={value}
          onValueChange={onChange}
          items={items}
        >
          <SelectTrigger className="w-40" aria-label={label}>
            <SelectValue placeholder={hideDefault ? t('home.selectPlaceholder') : undefined} />
          </SelectTrigger>
          <SelectContent>
            <SelectGroup>
              {!hideDefault && <SelectItem value="__default">{defaultLabel}</SelectItem>}
              {hideDefault && <SelectItem value="__none">{defaultLabel}</SelectItem>}
              {devices.map(d => (
                <SelectItem key={d.name} value={d.name}>
                  {d.name}
                </SelectItem>
              ))}
            </SelectGroup>
          </SelectContent>
        </Select>
      </div>
    </div>
  )
}
