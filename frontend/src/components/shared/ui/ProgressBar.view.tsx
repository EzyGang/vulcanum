import type { JSX } from 'preact';

interface ProgressBarProps {
  value: number;
  max: number;
  showFraction?: boolean;
}

const fillColor = (ratio: number): string => {
  if (ratio >= 1) return 'bg-error';
  if (ratio >= 0.5) return 'bg-warning';
  return 'bg-success';
};

export const ProgressBar = ({ value, max, showFraction }: ProgressBarProps): JSX.Element => {
  const clampedRatio = Math.max(0, Math.min(1, max > 0 ? value / max : 0));

  return (
    <div class='flex items-center gap-2'>
      <div class='w-10 h-1.5 bg-surface-raised border border-white/10'>
        <div
          class={`h-full ${fillColor(clampedRatio)}`}
          style={{ width: `${clampedRatio * 100}%` }}
        />
      </div>
      {showFraction && (
        <span class='text-text-muted text-xs font-mono'>
          {value} / {max}
        </span>
      )}
    </div>
  );
};
