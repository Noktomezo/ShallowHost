import { getVersion } from '@tauri-apps/api/app'
import { invoke } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { disable as disableAutostart, enable as enableAutostart } from '@tauri-apps/plugin-autostart'
import { Globe, Monitor, Moon, RefreshCw, Sun } from 'lucide-react'
import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { updateService } from '@/shared/lib/updater'
import { cn } from '@/shared/lib/utils'
import { useLanguageStore } from '@/shared/model/language-store'
import { useThemeStore } from '@/shared/model/theme-store'
import { useTrayStore } from '@/shared/model/tray-store'
import { useUpdateStore } from '@/shared/model/update-store'
import { Badge } from '@/shared/ui/badge'
import { Button } from '@/shared/ui/button'
import {
  Card,
  CardAction,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/shared/ui/card'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
} from '@/shared/ui/select'
import { showUpdateToast } from '@/shared/ui/sonner'
import { Switch } from '@/shared/ui/switch'

function FlagRU() {
  return (
    <svg className="size-4 shrink-0 rounded-xs border border-border/20" viewBox="0 0 9 6" data-icon="inline-start">
      <rect fill="#fff" width="9" height="2" y="0" />
      <rect fill="#0039a6" width="9" height="2" y="2" />
      <rect fill="#d52b1e" width="9" height="2" y="4" />
    </svg>
  )
}

function FlagEN() {
  return (
    <svg className="size-4 shrink-0 rounded-xs border border-border/20" viewBox="0 0 60 30" data-icon="inline-start">
      <clipPath id="s">
        <path d="M0,0 v30 h60 v-30 z" />
      </clipPath>
      <path d="M0,0 L60,30 M60,0 L0,30" stroke="#00247d" strokeWidth="6" clipPath="url(#s)" />
      <path d="M0,0 L60,30 M60,0 L0,30" stroke="#fff" strokeWidth="10" clipPath="url(#s)" />
      <path d="M0,0 L60,30 M60,0 L0,30" stroke="#cf142b" strokeWidth="6" clipPath="url(#s)" />
      <path d="M30,0 v30 M0,15 h60" stroke="#fff" strokeWidth="10" />
      <path d="M30,0 v30 M0,15 h60" stroke="#cf142b" strokeWidth="6" />
    </svg>
  )
}

export function SettingsPage() {
  const { t } = useTranslation()
  const theme = useThemeStore(s => s.theme)
  const setTheme = useThemeStore(s => s.setTheme)
  const language = useLanguageStore(s => s.language)
  const setLanguage = useLanguageStore(s => s.setLanguage)
  const check = useUpdateStore(s => s.checkResult)
  const setCheckResult = useUpdateStore(s => s.setCheckResult)
  const autoCheckEnabled = useUpdateStore(s => s.autoCheckEnabled)
  const setAutoCheckEnabled = useUpdateStore(s => s.setAutoCheckEnabled)
  const autostart = useTrayStore(s => s.autostart)
  const autostartToTray = useTrayStore(s => s.autostartToTray)
  const minimizeToTray = useTrayStore(s => s.minimizeToTray)
  const setAutostart = useTrayStore(s => s.setAutostart)
  const setAutostartToTray = useTrayStore(s => s.setAutostartToTray)
  const setMinimizeToTray = useTrayStore(s => s.setMinimizeToTray)
  const [version, setVersion] = useState('')

  useEffect(() => {
    getVersion().then(setVersion)
    const trayState = useTrayStore.getState()
    // ponytail: sync persisted settings with OS/Rust state on mount.
    invoke('set_close_to_tray', { enabled: trayState.minimizeToTray })
    if (trayState.autostart)
      enableAutostart().catch(() => {})
    // ponytail: if launched via autostart but autostartToTray is off, show window
    // (Rust hid it in setup because it can't read zustand state).
    invoke<boolean>('is_autostart_launch').then((launched) => {
      if (launched && !trayState.autostartToTray)
        getCurrentWindow().show()
    })
  }, [])

  async function handleAutostart(v: boolean) {
    setAutostart(v)
    if (v)
      await enableAutostart()
    else
      await disableAutostart()
  }

  function handleMinimizeToTray(v: boolean) {
    setMinimizeToTray(v)
    invoke('set_close_to_tray', { enabled: v })
  }

  async function handleCheck() {
    setCheckResult({ kind: 'checking' })
    try {
      const info = await updateService.check()
      if (info) {
        setCheckResult({ kind: 'available', info })
        showUpdateToast(info)
      }
      else {
        setCheckResult({ kind: 'up-to-date' })
      }
    }
    catch (e) {
      setCheckResult({ kind: 'up-to-date' })
      console.error('[update] check failed:', e)
    }
  }

  return (
    <div className="flex flex-col gap-0.5">
      <h1 className="text-xl font-semibold">{t('settings.title')}</h1>
      <p className="text-sm text-muted-foreground">
        {t('settings.description')}
      </p>

      <Card className="mt-3 w-full">
        <CardHeader className="gap-0.5">
          <CardTitle>{t('settings.theme')}</CardTitle>
          <CardDescription>{t('settings.themeDescription')}</CardDescription>
          <CardAction className="self-center">
            <Select
              value={theme}
              onValueChange={v => setTheme(v as 'system' | 'dark' | 'light')}
              items={{
                system: t('settings.themeSystem'),
                light: t('settings.themeLight'),
                dark: t('settings.themeDark'),
              }}
            >
              <SelectTrigger className="w-40">
                <span className="flex items-center gap-1.5 text-left flex-1">
                  {theme === 'system' && <Monitor className="size-4" />}
                  {theme === 'light' && <Sun className="size-4" />}
                  {theme === 'dark' && <Moon className="size-4" />}
                  {theme === 'system' && t('settings.themeSystem')}
                  {theme === 'light' && t('settings.themeLight')}
                  {theme === 'dark' && t('settings.themeDark')}
                </span>
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="system">
                  <Monitor className="size-4" data-icon="inline-start" />
                  {t('settings.themeSystem')}
                </SelectItem>
                <SelectItem value="light">
                  <Sun className="size-4" data-icon="inline-start" />
                  {t('settings.themeLight')}
                </SelectItem>
                <SelectItem value="dark">
                  <Moon className="size-4" data-icon="inline-start" />
                  {t('settings.themeDark')}
                </SelectItem>
              </SelectContent>
            </Select>
          </CardAction>
        </CardHeader>
      </Card>

      <Card className="mt-3 w-full">
        <CardHeader className="gap-0.5">
          <CardTitle>{t('settings.language')}</CardTitle>
          <CardDescription>{t('settings.languageDescription')}</CardDescription>
          <CardAction className="self-center">
            <Select
              value={language}
              onValueChange={v => setLanguage(v as 'system' | 'ru' | 'en')}
              items={{
                system: t('settings.langSystem'),
                ru: t('settings.langRu'),
                en: t('settings.langEn'),
              }}
            >
              <SelectTrigger className="w-40">
                <span className="flex items-center gap-1.5 text-left flex-1">
                  {language === 'system' && <Globe className="size-4" />}
                  {language === 'ru' && <FlagRU />}
                  {language === 'en' && <FlagEN />}
                  {language === 'system' && t('settings.langSystem')}
                  {language === 'ru' && t('settings.langRu')}
                  {language === 'en' && t('settings.langEn')}
                </span>
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="system">
                  <Globe className="size-4" data-icon="inline-start" />
                  {t('settings.langSystem')}
                </SelectItem>
                <SelectItem value="ru">
                  <FlagRU />
                  {t('settings.langRu')}
                </SelectItem>
                <SelectItem value="en">
                  <FlagEN />
                  {t('settings.langEn')}
                </SelectItem>
              </SelectContent>
            </Select>
          </CardAction>
        </CardHeader>
      </Card>

      <Card className="mt-3 w-full">
        <CardHeader className="gap-0.5">
          <CardTitle>{t('settings.system')}</CardTitle>
          <CardDescription>{t('settings.systemDescription')}</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex flex-col gap-4">
            <div className="flex items-center justify-between gap-2">
              <div className="flex flex-col gap-0">
                <span className="text-sm font-medium">{t('settings.autostart')}</span>
                <span className="text-xs text-muted-foreground">{t('settings.autostartDescription')}</span>
              </div>
              <Switch
                checked={autostart}
                onCheckedChange={v => handleAutostart(!!v)}
              />
            </div>
            <div className="flex items-center justify-between gap-2">
              <div className="flex flex-col gap-0">
                <span className="text-sm font-medium">{t('settings.autostartToTray')}</span>
                <span className="text-xs text-muted-foreground">{t('settings.autostartToTrayDescription')}</span>
              </div>
              <Switch
                checked={autostartToTray}
                disabled={!autostart}
                onCheckedChange={v => setAutostartToTray(!!v)}
              />
            </div>
            <div className="flex items-center justify-between gap-2">
              <div className="flex flex-col gap-0">
                <span className="text-sm font-medium">{t('settings.minimizeToTray')}</span>
                <span className="text-xs text-muted-foreground">{t('settings.minimizeToTrayDescription')}</span>
              </div>
              <Switch
                checked={minimizeToTray}
                onCheckedChange={v => handleMinimizeToTray(!!v)}
              />
            </div>
          </div>
        </CardContent>
      </Card>

      <Card className="mt-3 w-full">
        <CardHeader className="gap-0.5">
          <div className="flex items-center gap-2">
            <CardTitle>{t('settings.updates')}</CardTitle>
            {version && (
              <Badge variant="secondary">
                v
                {version}
              </Badge>
            )}
            {check.kind === 'up-to-date' && <Badge variant="green">{t('update.latest')}</Badge>}
            {check.kind === 'available' && <Badge variant="default">{t('update.newerAvailable')}</Badge>}
          </div>
          <CardDescription>{t('settings.updatesDescription')}</CardDescription>
          <CardAction className="self-center">
            <Button
              variant="outline"
              size="default"
              disabled={check.kind === 'checking'}
              onClick={handleCheck}
            >
              <RefreshCw
                className={cn(
                  'size-4',
                  check.kind === 'checking' && 'animate-spin',
                )}
              />
              {check.kind === 'checking'
                ? t('update.checking')
                : check.kind === 'available'
                  ? t('update.checkAgain')
                  : t('update.check')}
            </Button>
          </CardAction>
        </CardHeader>
        <CardContent>
          <div className="flex items-center justify-between gap-2">
            <div className="flex flex-col gap-0">
              <span className="text-sm font-medium">{t('settings.autoCheck')}</span>
              <span className="text-xs text-muted-foreground">{t('settings.autoCheckDescription')}</span>
            </div>
            <Switch
              checked={autoCheckEnabled}
              onCheckedChange={v => setAutoCheckEnabled(!!v)}
            />
          </div>
        </CardContent>
      </Card>
    </div>
  )
}
