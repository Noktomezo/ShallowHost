import type { AudioConfig } from '@/shared/model/audio-config-store'
import { invoke } from '@tauri-apps/api/core'
import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import {
  Card,
  CardAction,
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
import { VolumeMeter } from '@/shared/ui/VolumeMeter'

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

function formatChannelPair(ch1: string, ch2: string): string {
  if (!ch1 || !ch2)
    return ch1 || ch2 || ''
  if (ch1 === ch2)
    return ch1

  let prefixLen = 0
  const minLen = Math.min(ch1.length, ch2.length)
  while (prefixLen < minLen && ch1[prefixLen] === ch2[prefixLen]) {
    prefixLen++
  }

  while (
    prefixLen > 0
    && /\d/.test(ch1[prefixLen - 1])
    && prefixLen < ch1.length
    && /\d/.test(ch1[prefixLen])
  ) {
    prefixLen--
  }

  const prefix = ch1.slice(0, prefixLen)
  let rem1 = ch1.slice(prefixLen)
  let rem2 = ch2.slice(prefixLen)

  let suffixLen = 0
  const minRemLen = Math.min(rem1.length, rem2.length)
  while (
    suffixLen < minRemLen
    && rem1[rem1.length - 1 - suffixLen] === rem2[rem2.length - 1 - suffixLen]
  ) {
    suffixLen++
  }

  while (
    suffixLen > 0
    && /\d/.test(rem1[rem1.length - suffixLen])
    && rem1.length - suffixLen > 0
    && /\d/.test(rem1[rem1.length - 1 - suffixLen])
  ) {
    suffixLen--
  }

  const suffix = rem1.slice(rem1.length - suffixLen)
  rem1 = rem1.slice(0, rem1.length - suffixLen)
  rem2 = rem2.slice(0, rem2.length - suffixLen)

  if (rem1 && rem2) {
    return `${prefix}${rem1} + ${rem2}${suffix}`
  }
  return `${ch1} / ${ch2}`
}

function groupChannels(channels: string[]): { label: string, indices: number[] }[] {
  const pairs: { label: string, indices: number[] }[] = []
  if (!channels)
    return pairs
  for (let i = 0; i < channels.length; i += 2) {
    if (i + 1 < channels.length) {
      pairs.push({
        label: formatChannelPair(channels[i], channels[i + 1]),
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

function toggleChannelIndices(active: number[], indices: number[], checked: boolean) {
  const next = new Set(active)
  if (checked) {
    indices.forEach(idx => next.add(idx))
  }
  else {
    indices.forEach(idx => next.delete(idx))
  }
  return [...next]
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
  const [levels, setLevels] = useState({ input: 0, output: 0 })

  useEffect(() => {
    let mounted = true
    let inFlight = false
    const timer = setInterval(async () => {
      if (inFlight)
        return
      inFlight = true
      try {
        const res = await invoke<{ input: number, output: number }>('get_audio_levels')
        if (mounted)
          setLevels(res)
      }
      catch {}
      finally {
        inFlight = false
      }
    }, 30)
    return () => {
      mounted = false
      clearInterval(timer)
    }
  }, [])

  const inputPairs = groupChannels(devices.input_channels || [])
  const outputPairs = groupChannels(devices.output_channels || [])

  const activeInputs = config.active_inputs ?? [0, 1]
  const activeOutputs = config.active_outputs ?? [0, 1]
  const activeInputsSet = new Set(activeInputs)
  const activeOutputsSet = new Set(activeOutputs)

  const handleInputToggle = (indices: number[], checked: boolean) => {
    updateConfig({ active_inputs: toggleChannelIndices(activeInputs, indices, checked) })
  }

  const handleOutputToggle = (indices: number[], checked: boolean) => {
    updateConfig({ active_outputs: toggleChannelIndices(activeOutputs, indices, checked) })
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

  const isOutputActive = config.output_device && config.output_device !== '__none'
  const isInputActive = config.input_device && config.input_device !== '__none'

  return (
    <Card className="w-full">
      <CardHeader className="gap-0.5">
        <CardTitle>{t('home.audio')}</CardTitle>
        <CardDescription>{t('home.audioDescription')}</CardDescription>
        <CardAction className="self-center">
          {/* Custom sliding toggle between Stereo and Mono */}
          <div className="relative inline-flex items-center rounded-md bg-muted/60 p-0 border border-border/40 select-none text-xs font-semibold h-8 w-40 overflow-hidden shrink-0">
            {/* Moving background thumb */}
            <div
              className={`absolute top-0 bottom-0 left-0 rounded-[calc(var(--radius)-1px)] transition-all duration-300 ease-in-out w-[79px] ${
                config.is_mono
                  ? 'translate-x-[79px] bg-purple shadow-sm shadow-purple/20'
                  : 'translate-x-0 bg-primary shadow-sm shadow-primary/20'
              }`}
            />
            {/* Stereo Label */}
            <button
              type="button"
              onClick={(e) => {
                e.stopPropagation()
                updateConfig({ is_mono: false })
              }}
              className={`relative z-10 flex-1 text-center h-full flex items-center justify-center cursor-pointer transition-colors duration-300 ${
                !config.is_mono ? 'text-primary-foreground font-semibold' : 'text-muted-foreground hover:text-foreground/80'
              }`}
            >
              {t('home.stereo')}
            </button>
            {/* Mono Label */}
            <button
              type="button"
              onClick={(e) => {
                e.stopPropagation()
                updateConfig({ is_mono: true })
              }}
              className={`relative z-10 flex-1 text-center h-full flex items-center justify-center cursor-pointer transition-colors duration-300 ${
                config.is_mono ? 'text-white' : 'text-muted-foreground hover:text-foreground/80'
              }`}
            >
              {t('home.mono')}
            </button>
          </div>
        </CardAction>
      </CardHeader>
      <Separator />
      <CardContent>
        <div className="flex flex-col gap-4">
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
                                <span className="text-xs text-muted-foreground">No channels available</span>
                              )
                            : (
                                outputPairs.map((p) => {
                                  const isChecked = p.indices.every(i => activeOutputsSet.has(i))
                                  return (
                                    <label key={p.label} className="flex items-center justify-between gap-2 text-sm select-none cursor-pointer p-0.5 rounded hover:bg-muted/40 overflow-hidden">
                                      <div className="flex items-center gap-2 min-w-0 flex-1">
                                        <Checkbox
                                          checked={isChecked}
                                          onCheckedChange={checked => handleOutputToggle(p.indices, !!checked)}
                                        />
                                        <span className="truncate">{p.label}</span>
                                      </div>
                                    </label>
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
                                <span className="text-xs text-muted-foreground">No channels available</span>
                              )
                            : (
                                inputPairs.map((p) => {
                                  const isChecked = p.indices.every(i => activeInputsSet.has(i))
                                  return (
                                    <label key={p.label} className="flex items-center justify-between gap-2 text-sm select-none cursor-pointer p-0.5 rounded hover:bg-muted/40 overflow-hidden">
                                      <div className="flex items-center gap-2 min-w-0 flex-1">
                                        <Checkbox
                                          checked={isChecked}
                                          onCheckedChange={checked => handleInputToggle(p.indices, !!checked)}
                                        />
                                        <span className="truncate">{p.label}</span>
                                      </div>
                                    </label>
                                  )
                                })
                              )}
                        </div>
                      </div>
                    </div>
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
                    meter={<VolumeMeter level={isOutputActive ? levels.output : 0} />}
                  />

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
                    meter={<VolumeMeter level={isInputActive ? levels.input : 0} />}
                  />
                </>
              )}

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
