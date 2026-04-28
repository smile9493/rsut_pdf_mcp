import { ref, watch } from 'vue'
import type { Ref } from 'vue'

export const THEMES = {
  LIGHT: 'light',
  DARK: 'dark',
  AUTO: 'auto'
} as const

export type ThemeValue = typeof THEMES[keyof typeof THEMES]

const getSavedTheme = (): ThemeValue => {
  const saved = localStorage.getItem('pdf-module-theme')
  if (saved && Object.values(THEMES).includes(saved as ThemeValue)) {
    return saved as ThemeValue
  }
  return THEMES.DARK
}

const themeSetting: Ref<ThemeValue> = ref(getSavedTheme())
const appliedTheme: Ref<'light' | 'dark'> = ref(THEMES.DARK)

const computeAppliedTheme = (): 'light' | 'dark' => {
  if (themeSetting.value === THEMES.AUTO) {
    const isDark = window.matchMedia('(prefers-color-scheme: dark)').matches
    return isDark ? THEMES.DARK : THEMES.LIGHT
  }
  return themeSetting.value as 'light' | 'dark'
}

const applyTheme = (theme: 'light' | 'dark'): void => {
  const root = document.documentElement

  if (theme === THEMES.DARK) {
    root.classList.add('dark')
    root.classList.remove('light')
  } else {
    root.classList.add('light')
    root.classList.remove('dark')
  }

  appliedTheme.value = theme
}

export const setTheme = (theme: ThemeValue): void => {
  if (!Object.values(THEMES).includes(theme)) {
    return
  }

  themeSetting.value = theme
  localStorage.setItem('pdf-module-theme', theme)

  const actualTheme = computeAppliedTheme()
  applyTheme(actualTheme)
}

export const getTheme = (): ThemeValue => {
  return themeSetting.value
}

export const getAppliedTheme = (): 'light' | 'dark' => {
  return appliedTheme.value
}

export const initTheme = (): void => {
  const actualTheme = computeAppliedTheme()
  applyTheme(actualTheme)

  window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', (e: MediaQueryListEvent) => {
    if (themeSetting.value === THEMES.AUTO) {
      applyTheme(e.matches ? THEMES.DARK : THEMES.LIGHT)
    }
  })
}

watch(themeSetting, () => {
  const actualTheme = computeAppliedTheme()
  applyTheme(actualTheme)
})

export default {
  themeSetting,
  appliedTheme,
  setTheme,
  getTheme,
  getAppliedTheme,
  initTheme,
  THEMES
}
