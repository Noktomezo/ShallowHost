import { Checkbox as CheckboxPrimitive } from '@base-ui/react/checkbox'
import { Check } from 'lucide-react'

import { cn } from '@/shared/lib/utils'

function Checkbox({
  className,
  ...props
}: CheckboxPrimitive.Root.Props) {
  return (
    <CheckboxPrimitive.Root
      data-slot="checkbox"
      className={cn(
        'peer relative inline-flex size-4 shrink-0 items-center justify-center rounded border border-input text-primary outline-none transition-all focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50 disabled:cursor-not-allowed disabled:opacity-50 data-checked:bg-primary data-checked:text-primary-foreground data-checked:border-primary data-unchecked:bg-background',
        className,
      )}
      {...props}
    >
      <CheckboxPrimitive.Indicator className="flex items-center justify-center text-current">
        <Check className="size-3.5 stroke-[3]" />
      </CheckboxPrimitive.Indicator>
    </CheckboxPrimitive.Root>
  )
}

export { Checkbox }
