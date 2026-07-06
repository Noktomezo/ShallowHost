import type { AudioConfig } from '@/shared/model/audio-config-store'
import { useTranslation } from 'react-i18next'
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/shared/ui/card'
import { Checkbox } from '@/shared/ui/checkbox'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/shared/ui/select'
import { Separator } from '@/shared/ui/separator'

interface DeviceInfo {
  name: string
  default: boolean
}

interface AudioDevices {
  inputs: DeviceInfo[]
  outputs: DeviceInfo[]
  input_channels?: string[] | null
  output_channels?: string[] | null
}

function groupChannels(channels: string[]): { label: string, indices: number[] }[] {
  const pairs: { label: string, indices: number[] }[] = []
  if (!channels)
    return pairs
  for (let i = 0; i < channels.length; i += 2) {
    if (i + 1 < channels.length) {
      pairs.push({
        label: `${channels[i]} + ${channels[i + 1]}`,
        indices: [i, i + 1],
      })
    }
    else {
      pairs.push({
        label: channels[i],
        indices: [i],
      })
    }
  }
  return pairs
}

const SAMPLE_RATES = [44100, 48000, 88200, 96000, 192000]
const BUFFER_SIZES = [8, 16, 32, 64, 128, 256, 512, 1024, 2048]
const DRIVER_ITEMS = { wasapi: 'WASAPI', asio: 'ASIO' }

function DeviceSelect({
  label,
  description,
  value,
  items,
  devices,
  onChange,
  defaultLabel,
  hideDefault = false,
}: {
  label: string
  description: string
  value: string
  items: Record<string, React.ReactNode>
  devices: DeviceInfo[]
  onChange: (v: string | null) => void
  defaultLabel: string
  hideDefault?: boolean
}) {
  return (
    <div className="flex items-center justify-between gap-2">
      <div className="flex flex-col gap-0">
        <span className="text-sm font-medium">{label}</span>
        <span className="text-xs text-muted-foreground">{description}</span>
      </div>
      <Select
        value={value}
        onValueChange={onChange}
        items={items}
      >
        <SelectTrigger className="w-40">
          <SelectValue placeholder={hideDefault ? 'Select...' : undefined} />
        </SelectTrigger>
        <SelectContent>
          {!hideDefault && <SelectItem value="__default">{defaultLabel}</SelectItem>}
          {hideDefault && <SelectItem value="__none">{defaultLabel}</SelectItem>}
          {devices.map(d => (
            <SelectItem key={d.name} value={d.name}>
              {d.name}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
    </div>
  )
}

export function AudioConfigCard({
  config,
  devices,
  updateConfig,
}: {
  config: AudioConfig
  devices: AudioDevices
  updateConfig: (patch: Partial<AudioConfig>) => void
}) {
  const { t } = useTranslation()

  const inputPairs = groupChannels(devices.input_channels || [])
  const outputPairs = groupChannels(devices.output_channels || [])

  const activeInputs = config.active_inputs ?? [0, 1]
  const activeOutputs = config.active_outputs ?? [0, 1]

  const handleInputToggle = (indices: number[], checked: boolean) => {
    let next = [...activeInputs]
    if (checked) {
      indices.forEach((idx) => {
        if (!next.includes(idx))
          next.push(idx)
      })
    }
    else {
      next = next.filter(idx => !indices.includes(idx))
    }
    updateConfig({ active_inputs: next })
  }

  const handleOutputToggle = (indices: number[], checked: boolean) => {
    let next = [...activeOutputs]
    if (checked) {
      indices.forEach((idx) => {
        if (!next.includes(idx))
          next.push(idx)
      })
    }
    else {
      next = next.filter(idx => !indices.includes(idx))
    }
    updateConfig({ active_outputs: next })
  }

  const outputItems = Object.fromEntries([
    ['__none', t('home.noneDevice')],
    ...devices.outputs.map(d => [d.name, d.name]),
  ])
  const inputItems = Object.fromEntries([
    ['__none', t('home.noneDevice')],
    ...devices.inputs.map(d => [d.name, d.name]),
  ])
  const rateItems = Object.fromEntries(
    SAMPLE_RATES.map(r => [
      String(r),
      r >= 1000 ? `${r / 1000} kHz` : `${r} Hz`,
    ]),
  )
  const bufferItems = Object.fromEntries(
    BUFFER_SIZES.map(b => [
      String(b),
      <>
        {b}
        <span className="ml-1 text-muted-foreground">
          (
          {(b / config.sample_rate * 1000).toFixed(1)}
          {' '}
          ms)
        </span>
      </>,
    ]),
  )

  return (
    <Card className="w-full">
      <CardHeader className="flex flex-row items-center justify-between gap-4 space-y-0 pb-5">
        <div className="flex flex-col gap-0.5">
          <CardTitle>{t('home.audio')}</CardTitle>
          <CardDescription>{t('home.audioDescription')}</CardDescription>
        </div>

        {/* Custom sliding toggle between Stereo and Mono */}
        <div className="relative inline-flex items-center rounded-md bg-muted/60 p-1 border border-border/40 select-none text-xs font-semibold h-9 w-40 overflow-hidden shrink-0">
          {/* Moving background thumb */}
          <div
            className={`absolute top-1 bottom-1 left-1 rounded-sm transition-all duration-300 ease-in-out w-[72px] ${
              config.mono
                ? 'translate-x-[76px] bg-violet-600 shadow-sm shadow-violet-600/20'
                : 'translate-x-0 bg-background border border-border/40 shadow-sm'
            }`}
          />
          {/* Stereo Label */}
          <button
            type="button"
            onClick={(e) => {
              e.stopPropagation()
              updateConfig({ mono: false })
            }}
            className={`relative z-10 flex-1 text-center h-full flex items-center justify-center cursor-pointer transition-colors duration-300 ${
              !config.mono ? 'text-foreground font-semibold' : 'text-muted-foreground hover:text-foreground/80'
            }`}
          >
            {t('home.stereo')}
          </button>
          {/* Mono Label */}
          <button
            type="button"
            onClick={(e) => {
              e.stopPropagation()
              updateConfig({ mono: true })
            }}
            className={`relative z-10 flex-1 text-center h-full flex items-center justify-center cursor-pointer transition-colors duration-300 ${
              config.mono ? 'text-white' : 'text-muted-foreground hover:text-foreground/80'
            }`}
          >
            {t('home.mono')}
          </button>
        </div>
      </CardHeader>
      <CardContent>
        <div className="flex flex-col gap-2">
          <div className="flex items-center justify-between gap-2">
            <div className="flex flex-col gap-0">
              <span className="text-sm font-medium">{t('home.driver')}</span>
              <span className="text-xs text-muted-foreground">
                {t('home.driverDescription')}
              </span>
            </div>
            <Select
              value={config.driver}
              onValueChange={v => updateConfig({ driver: v as string })}
              items={DRIVER_ITEMS}
            >
              <SelectTrigger className="w-40">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="wasapi">WASAPI</SelectItem>
                <SelectItem value="asio">
                  ASIO
                </SelectItem>
              </SelectContent>
            </Select>
          </div>

          <Separator />

          {config.driver === 'asio'
            ? (
                <>
                  <DeviceSelect
                    label={t('home.device')}
                    description={t('home.deviceDescription')}
                    value={config.output_device ?? '__none'}
                    items={outputItems}
                    devices={devices.outputs}
                    onChange={(v) => {
                      updateConfig({
                        input_device: v,
                        output_device: v,
                        active_inputs: null,
                        active_outputs: null,
                      })
                    }}
                    defaultLabel={t('home.noneDevice')}
                    hideDefault={true}
                  />

                  {config.output_device && config.output_device !== '__none' && (
                    <>
                      <Separator />
                      <div className="grid grid-cols-2 gap-4">
                        <div className="flex flex-col gap-2">
                          <span className="text-sm font-medium">
                            {t('home.activeOutputChannels')}
                            :
                          </span>
                          <div className="flex flex-col gap-1.5 rounded-md border border-input p-3 bg-muted/20 max-h-40 overflow-y-auto">
                            {outputPairs.length === 0
                              ? (
                                  <span className="text-xs text-muted-foreground">No channels available</span>
                                )
                              : (
                                  outputPairs.map((p, idx) => {
                                    const isChecked = p.indices.every(i => activeOutputs.includes(i))
                                    return (
                                      <label key={idx} className="flex items-center gap-2 text-sm select-none cursor-pointer">
                                        <Checkbox
                                          checked={isChecked}
                                          onCheckedChange={checked => handleOutputToggle(p.indices, !!checked)}
                                        />
                                        <span>{p.label}</span>
                                      </label>
                                    )
                                  })
                                )}
                          </div>
                        </div>

                        <div className="flex flex-col gap-2">
                          <span className="text-sm font-medium">
                            {t('home.activeInputChannels')}
                            :
                          </span>
                          <div className="flex flex-col gap-1.5 rounded-md border border-input p-3 bg-muted/20 max-h-40 overflow-y-auto">
                            {inputPairs.length === 0
                              ? (
                                  <span className="text-xs text-muted-foreground">No channels available</span>
                                )
                              : (
                                  inputPairs.map((p, idx) => {
                                    const isChecked = p.indices.every(i => activeInputs.includes(i))
                                    return (
                                      <label key={idx} className="flex items-center gap-2 text-sm select-none cursor-pointer">
                                        <Checkbox
                                          checked={isChecked}
                                          onCheckedChange={checked => handleInputToggle(p.indices, !!checked)}
                                        />
                                        <span>{p.label}</span>
                                      </label>
                                    )
                                  })
                                )}
                          </div>
                        </div>
                      </div>
                    </>
                  )}
                </>
              )
            : (
                <>
                  <DeviceSelect
                    label={t('home.outputDevice')}
                    description={t('home.outputDeviceDescription')}
                    value={config.output_device ?? '__none'}
                    items={outputItems}
                    devices={devices.outputs}
                    onChange={v =>
                      updateConfig({ output_device: v })}
                    defaultLabel={t('home.noneDevice')}
                    hideDefault={true}
                  />

                  <Separator />

                  <DeviceSelect
                    label={t('home.inputDevice')}
                    description={t('home.inputDeviceDescription')}
                    value={config.input_device ?? '__none'}
                    items={inputItems}
                    devices={devices.inputs}
                    onChange={v =>
                      updateConfig({ input_device: v })}
                    defaultLabel={t('home.noneDevice')}
                    hideDefault={true}
                  />
                </>
              )}

          <Separator />

          <div className="flex items-center justify-between gap-2">
            <div className="flex flex-col gap-0">
              <span className="text-sm font-medium">{t('home.sampleRate')}</span>
              <span className="text-xs text-muted-foreground">
                {t('home.sampleRateDescription')}
              </span>
            </div>
            <Select
              value={String(config.sample_rate)}
              onValueChange={v => updateConfig({ sample_rate: Number(v) })}
              items={rateItems}
            >
              <SelectTrigger className="w-40">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {SAMPLE_RATES.map(r => (
                  <SelectItem key={r} value={String(r)}>
                    {r >= 1000 ? `${r / 1000} kHz` : `${r} Hz`}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          <Separator />

          <div className="flex items-center justify-between gap-2">
            <div className="flex flex-col gap-0">
              <span className="text-sm font-medium">{t('home.bufferSize')}</span>
              <span className="text-xs text-muted-foreground">
                {t('home.bufferSizeDescription')}
              </span>
            </div>
            <Select
              value={String(config.buffer_size)}
              onValueChange={v => updateConfig({ buffer_size: Number(v) })}
              items={bufferItems}
            >
              <SelectTrigger className="w-40">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {BUFFER_SIZES.map(b => (
                  <SelectItem key={b} value={String(b)}>
                    {b}
                    <span className="ml-1 text-muted-foreground">
                      (
                      {(b / config.sample_rate * 1000).toFixed(1)}
                      {' '}
                      ms)
                    </span>
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        </div>
      </CardContent>
    </Card>
  )
}

export type { AudioDevices, DeviceInfo }
