import type { JSX } from 'preact';

type ThemeMode = 'system' | 'light' | 'dark';

interface ThemeToggleViewProps {
  currentMode: ThemeMode;
  onToggle: () => void;
}

const SystemIcon = (): JSX.Element => (
  <svg
    xmlns='http://www.w3.org/2000/svg'
    width='18'
    height='18'
    viewBox='0 0 24 24'
    fill='none'
    stroke='currentColor'
    stroke-width='2'
    stroke-linecap='round'
    stroke-linejoin='round'
    aria-hidden='true'
  >
    <title>System theme</title>
    <rect x='2' y='3' width='20' height='14' rx='2' ry='2' />
    <line x1='8' y1='21' x2='16' y2='21' />
    <line x1='12' y1='17' x2='12' y2='21' />
  </svg>
);

const SunIcon = (): JSX.Element => (
  <svg
    xmlns='http://www.w3.org/2000/svg'
    width='18'
    height='18'
    viewBox='0 0 24 24'
    fill='none'
    stroke='currentColor'
    stroke-width='2'
    stroke-linecap='round'
    stroke-linejoin='round'
    aria-hidden='true'
  >
    <title>Light theme</title>
    <circle cx='12' cy='12' r='5' />
    <line x1='12' y1='1' x2='12' y2='3' />
    <line x1='12' y1='21' x2='12' y2='23' />
    <line x1='4.22' y1='4.22' x2='5.64' y2='5.64' />
    <line x1='18.36' y1='18.36' x2='19.78' y2='19.78' />
    <line x1='1' y1='12' x2='3' y2='12' />
    <line x1='21' y1='12' x2='23' y2='12' />
    <line x1='4.22' y1='19.78' x2='5.64' y2='18.36' />
    <line x1='18.36' y1='5.64' x2='19.78' y2='4.22' />
  </svg>
);

const MoonIcon = (): JSX.Element => (
  <svg
    xmlns='http://www.w3.org/2000/svg'
    width='18'
    height='18'
    viewBox='0 0 24 24'
    fill='none'
    stroke='currentColor'
    stroke-width='2'
    stroke-linecap='round'
    stroke-linejoin='round'
    aria-hidden='true'
  >
    <title>Dark theme</title>
    <path d='M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z' />
  </svg>
);

const getIconForMode = (mode: ThemeMode): JSX.Element => {
  switch (mode) {
    case 'system':
      return <SystemIcon />;
    case 'light':
      return <SunIcon />;
    case 'dark':
      return <MoonIcon />;
  }
};

const getLabelForMode = (mode: ThemeMode): string => {
  switch (mode) {
    case 'system':
      return 'System';
    case 'light':
      return 'Light';
    case 'dark':
      return 'Dark';
  }
};

export const ThemeToggleView = (props: ThemeToggleViewProps): JSX.Element => {
  const icon = getIconForMode(props.currentMode);
  const label = getLabelForMode(props.currentMode);

  return (
    <button
      type='button'
      onClick={props.onToggle}
      class='p-2 text-text-secondary hover:text-text-primary hover:bg-bg-hover transition-colors duration-150'
      aria-label={`Theme: ${label}. Click to cycle.`}
      title={`Theme: ${label}`}
    >
      {icon}
    </button>
  );
};
