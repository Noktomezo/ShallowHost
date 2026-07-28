import { useEffect, useRef, useState } from 'react'
import { cn } from '@/shared/lib/utils'

interface VolumeMeterProps {
  level: number // linear peak amplitude (0.0 to 1.0+)
  className?: string
}

// Logarithmic dBFS thresholds mapped to normalized 0..1 scale [-60 dBFS to 0 dBFS]:
// Dot 1: > -60 dBFS (0.01) -> Green
// Dot 2: > -48 dBFS (0.20) -> Green
// Dot 3: > -36 dBFS (0.40) -> Green
// Dot 4: > -24 dBFS (0.60) -> Green
// Dot 5: > -18 dBFS (0.70) -> Green
// Dot 6: > -12 dBFS (0.80) -> Yellow/Amber (-12 dBFS to -6 dBFS)
// Dot 7: > -6 dBFS  (0.90) -> Red (-6 dBFS to 0 dBFS)
// Dot 8: >= 0 dBFS Peak Hold -> Red Peak Hold LED (0 dBFS peak / clipping)
const THRESHOLDS = [0.01, 0.20, 0.40, 0.60, 0.70, 0.80, 0.90]
const TOTAL_DOTS = 8

function rawToDbFS(rawLevel: number): number {
  if (rawLevel <= 0.00001)
    return -100
  return 20 * Math.log10(rawLevel)
}

function scaleLevel(rawLevel: number): number {
  if (rawLevel <= 0.001)
    return 0
  const db = rawToDbFS(rawLevel)
  const normalized = (db + 60) / 60
  return Math.min(1, Math.max(0, normalized))
}

function formatDbTooltip(rawLevel: number): string {
  if (rawLevel <= 0.001)
    return 'Audio level: < -60 dBFS'
  const db = Math.round(rawToDbFS(rawLevel))
  return `Audio level: ${db >= 0 ? `+${db}` : db} dBFS`
}

export function VolumeMeter({ level, className }: VolumeMeterProps) {
  const [displayLevel, setDisplayLevel] = useState(0)
  const [peakHold, setPeakHold] = useState(false)
  const peakTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null)
  const targetRef = useRef(0)

  useEffect(() => {
    const target = scaleLevel(level)
    targetRef.current = target

    if (level >= 0.99 || target >= THRESHOLDS[6]) {
      setPeakHold(true)
      if (peakTimerRef.current) {
        clearTimeout(peakTimerRef.current)
      }
      peakTimerRef.current = setTimeout(() => {
        setPeakHold(false)
      }, 2000)
    }
  }, [level])

  useEffect(() => {
    const timer = setInterval(() => {
      setDisplayLevel((prev) => {
        const tgt = targetRef.current
        if (prev <= tgt) {
          return tgt
        }
        const next = prev * 0.86
        return next < THRESHOLDS[0] ? 0 : next
      })
    }, 40)

    return () => {
      clearInterval(timer)
      if (peakTimerRef.current) {
        clearTimeout(peakTimerRef.current)
      }
    }
  }, [])

  return (
    <div
      className={cn('flex flex-row items-center gap-1 shrink-0 select-none', className)}
      title={formatDbTooltip(level)}
    >
      {Array.from({ length: TOTAL_DOTS }).map((_, idx) => {
        let isActive = false
        let activeColor = 'bg-emerald-500 shadow-sm shadow-emerald-500/50'

        if (idx < 7) {
          isActive = displayLevel >= THRESHOLDS[idx]
        }
        else {
          // 8th dot: Peak Hold red LED (stays lit for 2s after clipping/peak)
          isActive = peakHold
        }

        if (idx === 5) {
          activeColor = 'bg-amber-400 shadow-sm shadow-amber-400/50'
        }
        else if (idx >= 6) {
          activeColor = 'bg-rose-500 shadow-sm shadow-rose-500/50'
        }

        return (
          <span
            key={idx}
            className={cn(
              'h-2 w-2 rounded-full transition-colors duration-100',
              isActive ? activeColor : 'bg-muted-foreground/25',
            )}
          />
        )
      })}
    </div>
  )
}
