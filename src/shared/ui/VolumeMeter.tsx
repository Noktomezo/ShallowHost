import { useEffect, useRef, useState } from 'react'
import { cn } from '@/shared/lib/utils'

interface VolumeMeterProps {
  level: number // 0.0 to 1.0+
  className?: string
}

const THRESHOLDS = [0.05, 0.22, 0.38, 0.52, 0.65, 0.80, 0.92]
const TOTAL_DOTS = 8

function scaleLevel(rawLevel: number): number {
  if (rawLevel <= 0.005)
    return 0
  // Power scaling (x^0.35) maps linear peak to perceived VU soundcard scale
  return Math.min(1, Math.max(0, rawLevel ** 0.35))
}

export function VolumeMeter({ level, className }: VolumeMeterProps) {
  const [displayLevel, setDisplayLevel] = useState(0)
  const [peakHold, setPeakHold] = useState(false)
  const peakTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null)

  useEffect(() => {
    const target = scaleLevel(level)
    if (target === 0) {
      setDisplayLevel(0)
    }
    else {
      setDisplayLevel((prev) => {
        if (target >= prev) {
          return target
        }
        // Decay timeout hold (~350ms smooth falloff)
        const next = prev * 0.86
        return next < THRESHOLDS[0] ? 0 : next
      })
    }

    if (target >= THRESHOLDS[6]) {
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
    return () => {
      if (peakTimerRef.current) {
        clearTimeout(peakTimerRef.current)
      }
    }
  }, [])

  return (
    <div
      className={cn('flex flex-row items-center gap-1 shrink-0 select-none', className)}
      title={`Audio level: ${Math.round(Math.min(level, 1) * 100)}%`}
    >
      {Array.from({ length: TOTAL_DOTS }).map((_, idx) => {
        let isActive = false
        let activeColor = 'bg-emerald-500 shadow-sm shadow-emerald-500/50'

        if (idx < 7) {
          isActive = displayLevel >= THRESHOLDS[idx]
        }
        else {
          // 8th dot: Peak Hold red LED (stays lit for 2s after clipping)
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
