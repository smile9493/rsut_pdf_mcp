import { createI18n } from 'vue-i18n'
import en from './locales/en'
import zh from './locales/zh'

const messages = {
  en,
  zh
}

// 获取保存的语言或浏览器语言
const getSavedLanguage = () => {
  const saved = localStorage.getItem('pdf-module-language')
  if (saved && (saved === 'en' || saved === 'zh')) {
    return saved
  }
  
  // 检测浏览器语言
  const browserLang = navigator.language || navigator.userLanguage
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

// 切换语言的辅助函数
export const setLanguage = (lang) => {
  i18n.global.locale.value = lang
  localStorage.setItem('pdf-module-language', lang)
  document.documentElement.lang = lang
}

// 获取当前语言
export const getLanguage = () => {
  return i18n.global.locale.value
}
