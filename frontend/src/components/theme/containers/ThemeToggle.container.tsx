import type { JSX } from 'preact';
import { useCallback } from 'preact/hooks';
import { cycleThemeMode, themeModeSignal } from '../../../stores/window-theme.store';
import { ThemeToggleView } from '../ui/ThemeToggle.view';

export const ThemeToggleContainer = (): JSX.Element => {
  const handleToggle = useCallback(() => {
    cycleThemeMode();
  }, []);

  return <ThemeToggleView currentMode={themeModeSignal.value} onToggle={handleToggle} />;
};
