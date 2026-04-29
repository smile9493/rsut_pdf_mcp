import { createI18n } from 'vue-i18n'
import en from './locales/en'
import zh from './locales/zh'

const messages = {
  en,
  zh
}

type SupportedLocale = 'en' | 'zh'

const getSavedLanguage = (): SupportedLocale => {
  const saved = localStorage.getItem('pdf-module-language')
  if (saved === 'en' || saved === 'zh') {
    return saved
  }

  const browserLang = navigator.language
  if (browserLang.startsWith('zh')) {
    return 'zh'
  }

  return 'en'
}

const i18n = createI18n({
  legacy: false,
  locale: getSavedLanguage(),
  fallbackLocale: 'en',
  messages
})

export default i18n

export const setLanguage = (lang: SupportedLocale): void => {
  ;(i18n.global.locale as { value: SupportedLocale }).value = lang
  localStorage.setItem('pdf-module-language', lang)
  document.documentElement.lang = lang
}

export const getLanguage = (): SupportedLocale => {
  return (i18n.global.locale as { value: SupportedLocale }).value
}
