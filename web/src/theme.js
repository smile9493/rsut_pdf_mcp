import { ref, watch } from 'vue'

// 主题类型
export const THEMES = {
  LIGHT: 'light',
  DARK: 'dark',
  AUTO: 'auto'
}

// 获取保存的主题或默认主题
const getSavedTheme = () => {
  const saved = localStorage.getItem('pdf-module-theme')
  if (saved && Object.values(THEMES).includes(saved)) {
    return saved
  }
  return THEMES.DARK // 默认深色主题
}

// 当前主题设置
const themeSetting = ref(getSavedTheme())

// 实际应用的主题（light 或 dark）
const appliedTheme = ref(THEMES.DARK)

// 计算实际主题
const computeAppliedTheme = () => {
  if (themeSetting.value === THEMES.AUTO) {
    // 检测系统主题
    const isDark = window.matchMedia('(prefers-color-scheme: dark)').matches
    return isDark ? THEMES.DARK : THEMES.LIGHT
  }
  return themeSetting.value
}

// 应用主题到 DOM
const applyTheme = (theme) => {
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

// 切换主题
export const setTheme = (theme) => {
  if (!Object.values(THEMES).includes(theme)) {
    return
  }
  
  themeSetting.value = theme
  localStorage.setItem('pdf-module-theme', theme)
  
  const actualTheme = computeAppliedTheme()
  applyTheme(actualTheme)
}

// 获取当前主题设置
export const getTheme = () => {
  return themeSetting.value
}

// 获取实际应用的主题
export const getAppliedTheme = () => {
  return appliedTheme.value
}

// 初始化主题
export const initTheme = () => {
  const actualTheme = computeAppliedTheme()
  applyTheme(actualTheme)
  
  // 监听系统主题变化
  window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', (e) => {
    if (themeSetting.value === THEMES.AUTO) {
      applyTheme(e.matches ? THEMES.DARK : THEMES.LIGHT)
    }
  })
}

// 监听主题设置变化
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
