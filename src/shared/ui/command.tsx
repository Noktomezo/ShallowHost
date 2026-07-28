import * as React from 'react'
import { useTranslation } from 'react-i18next'
import { Dialog, DialogContent, DialogDescription, DialogTitle } from '@/shared/ui/dialog'

export function CommandDialog({
  open,
  onOpenChange,
  children,
}: {
  open: boolean
  onOpenChange: (open: boolean) => void
  children: React.ReactNode
}) {
  const { t } = useTranslation()
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="p-0 sm:max-w-md overflow-hidden">
        <DialogTitle className="sr-only">{t('command.title')}</DialogTitle>
        <DialogDescription className="sr-only">{t('command.description')}</DialogDescription>
        {children}
      </DialogContent>
    </Dialog>
  )
}

export function CommandInput({
  value,
  onValueChange,
  placeholder,
}: {
  value?: string
  onValueChange?: (value: string) => void
  placeholder?: string
}) {
  const { t } = useTranslation()
  return (
    <div className="flex flex-col border-b border-border px-3 py-2.5">
      <input
        type="text"
        aria-label={t('command.inputAria')}
        className="w-full bg-transparent text-sm focus:outline-hidden focus-visible:ring-1 focus-visible:ring-ring rounded px-1 placeholder:text-muted-foreground"
        placeholder={placeholder ?? t('command.searchPlaceholder')}
        value={value}
        onChange={e => onValueChange?.(e.target.value)}
      />
    </div>
  )
}

export function CommandList({ children }: { children: React.ReactNode }) {
  return <div className="max-h-64 overflow-y-auto p-2">{children}</div>
}

export function CommandEmpty({ children }: { children: React.ReactNode }) {
  return <div className="p-4 text-center text-xs text-muted-foreground">{children}</div>
}

export function CommandItem({
  children,
  onSelect,
}: {
  children: React.ReactNode
  onSelect?: () => void
}) {
  return (
    <button
      type="button"
      className="flex w-full cursor-pointer items-center gap-2 rounded-md px-2.5 py-2 text-left text-sm hover:bg-accent hover:text-accent-foreground"
      onClick={onSelect}
    >
      {children}
    </button>
  )
}
