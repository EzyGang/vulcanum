import { effect, signal } from '@preact/signals';

type ThemeMode = 'system' | 'light' | 'dark';
type EffectiveTheme = 'light' | 'dark';

const STORAGE_KEY = 'vulcanum-theme';

const getSystemTheme = (): EffectiveTheme => {
  return window.matchMedia('(prefers-color-scheme: dark)')?.matches ? 'dark' : 'light';
};

const getInitialMode = (): ThemeMode => {
  const stored = localStorage.getItem(STORAGE_KEY);
  if (stored === 'light' || stored === 'dark' || stored === 'system') {
    return stored;
  }
  return 'system';
};

const computeEffectiveTheme = (mode: ThemeMode): EffectiveTheme => {
  if (mode === 'system') {
    return getSystemTheme();
  }
  return mode;
};

export const themeModeSignal = signal<ThemeMode>(getInitialMode());
export const effectiveThemeSignal = signal<EffectiveTheme>(computeEffectiveTheme(getInitialMode()));

let systemListener: ((event: MediaQueryListEvent) => void) | null = null;

const setupSystemListener = () => {
  if (systemListener) return;

  systemListener = (event: MediaQueryListEvent) => {
    if (themeModeSignal.value === 'system') {
      effectiveThemeSignal.value = event.matches ? 'dark' : 'light';
    }
  };

  window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', systemListener);
};

const removeSystemListener = () => {
  if (systemListener) {
    window.matchMedia('(prefers-color-scheme: dark)').removeEventListener('change', systemListener);
    systemListener = null;
  }
};

export const cycleThemeMode = (): void => {
  const current = themeModeSignal.value;
  const next: ThemeMode = current === 'system' ? 'light' : current === 'light' ? 'dark' : 'system';
  themeModeSignal.value = next;
  localStorage.setItem(STORAGE_KEY, next);
};

effect(() => {
  const mode = themeModeSignal.value;
  const effective = computeEffectiveTheme(mode);
  effectiveThemeSignal.value = effective;

  document.documentElement.setAttribute('data-theme', effective);

  if (mode === 'system') {
    setupSystemListener();
  } else {
    removeSystemListener();
  }
});

setupSystemListener();
