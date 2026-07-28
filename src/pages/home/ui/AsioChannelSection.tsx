import { useTranslation } from 'react-i18next'
import { Checkbox } from '@/shared/ui/checkbox'
import { VolumeMeter } from '@/shared/ui/VolumeMeter'

interface ChannelPair {
  label: string
  indices: number[]
}

interface AsioChannelSectionProps {
  outputPairs: ChannelPair[]
  inputPairs: ChannelPair[]
  activeOutputsSet: Set<number>
  activeInputsSet: Set<number>
  isOutputActive: boolean
  isInputActive: boolean
  levels: { input: number, output: number }
  handleOutputToggle: (indices: number[], checked: boolean) => void
  handleInputToggle: (indices: number[], checked: boolean) => void
}

export function AsioChannelSection({
  outputPairs,
  inputPairs,
  activeOutputsSet,
  activeInputsSet,
  isOutputActive,
  isInputActive,
  levels,
  handleOutputToggle,
  handleInputToggle,
}: AsioChannelSectionProps) {
  const { t } = useTranslation()

  return (
    <div className="grid grid-cols-2 gap-4">
      <div className="flex flex-col gap-2">
        <div className="flex items-center justify-between">
          <span className="text-sm font-medium">
            {t('home.activeOutputChannels')}
            :
          </span>
          <VolumeMeter level={isOutputActive ? levels.output : 0} />
        </div>
        <div className="flex flex-col gap-1.5 rounded-md border border-input p-3 bg-muted/20 max-h-40 overflow-y-auto">
          {outputPairs.length === 0
            ? (
                <span className="text-xs text-muted-foreground">{t('home.noChannelsAvailable')}</span>
              )
            : (
                outputPairs.map((p) => {
                  const isChecked = p.indices.every(i => activeOutputsSet.has(i))
                  return (
                    <div
                      key={p.label}
                      role="checkbox"
                      aria-checked={isChecked}
                      tabIndex={0}
                      className="flex items-center justify-between gap-2 text-sm select-none cursor-pointer p-0.5 rounded hover:bg-muted/40 overflow-hidden"
                      onClick={() => handleOutputToggle(p.indices, !isChecked)}
                      onKeyDown={(e) => {
                        if (e.key === ' ' || e.key === 'Enter') {
                          e.preventDefault()
                          handleOutputToggle(p.indices, !isChecked)
                        }
                      }}
                    >
                      <div className="flex items-center gap-2 min-w-0 flex-1">
                        <Checkbox
                          checked={isChecked}
                          tabIndex={-1}
                          aria-hidden="true"
                          className="pointer-events-none"
                        />
                        <span className="truncate">{p.label}</span>
                      </div>
                    </div>
                  )
                })
              )}
        </div>
      </div>

      <div className="flex flex-col gap-2">
        <div className="flex items-center justify-between">
          <span className="text-sm font-medium">
            {t('home.activeInputChannels')}
            :
          </span>
          <VolumeMeter level={isInputActive ? levels.input : 0} />
        </div>
        <div className="flex flex-col gap-1.5 rounded-md border border-input p-3 bg-muted/20 max-h-40 overflow-y-auto">
          {inputPairs.length === 0
            ? (
                <span className="text-xs text-muted-foreground">{t('home.noChannelsAvailable')}</span>
              )
            : (
                inputPairs.map((p) => {
                  const isChecked = p.indices.every(i => activeInputsSet.has(i))
                  return (
                    <div
                      key={p.label}
                      role="checkbox"
                      aria-checked={isChecked}
                      tabIndex={0}
                      className="flex items-center justify-between gap-2 text-sm select-none cursor-pointer p-0.5 rounded hover:bg-muted/40 overflow-hidden"
                      onClick={() => handleInputToggle(p.indices, !isChecked)}
                      onKeyDown={(e) => {
                        if (e.key === ' ' || e.key === 'Enter') {
                          e.preventDefault()
                          handleInputToggle(p.indices, !isChecked)
                        }
                      }}
                    >
                      <div className="flex items-center gap-2 min-w-0 flex-1">
                        <Checkbox
                          checked={isChecked}
                          tabIndex={-1}
                          aria-hidden="true"
                          className="pointer-events-none"
                        />
                        <span className="truncate">{p.label}</span>
                      </div>
                    </div>
                  )
                })
              )}
        </div>
      </div>
    </div>
  )
}
