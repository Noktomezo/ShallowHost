import { Globe, Monitor, Moon, Sun } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { useLanguageStore } from '@/shared/model/language-store'
import { useThemeStore } from '@/shared/model/theme-store'
import {
  Card,
  CardAction,
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
    </div>
  )
}
