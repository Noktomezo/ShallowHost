import { Monitor, Moon, Sun } from 'lucide-react'
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
  SelectValue,
} from '@/shared/ui/select'

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
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="system">
                  <Monitor data-icon="inline-start" />
                  {t('settings.themeSystem')}
                </SelectItem>
                <SelectItem value="light">
                  <Sun data-icon="inline-start" />
                  {t('settings.themeLight')}
                </SelectItem>
                <SelectItem value="dark">
                  <Moon data-icon="inline-start" />
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
              onValueChange={v => setLanguage(v as 'ru' | 'en')}
              items={{
                ru: t('settings.langRu'),
                en: t('settings.langEn'),
              }}
            >
              <SelectTrigger className="w-40">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="ru">
                  {t('settings.langRu')}
                </SelectItem>
                <SelectItem value="en">
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
