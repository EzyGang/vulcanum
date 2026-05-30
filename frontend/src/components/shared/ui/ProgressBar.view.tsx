import { Progress } from '@base-ui/react/progress';
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
  const clampedValue = Math.max(0, Math.min(max, max > 0 ? value : 0));

  return (
    <div class='flex items-center gap-2'>
      <Progress.Root value={clampedValue} max={max} className='w-10'>
        <Progress.Track className='h-1.5 bg-bg-active border border-border-base block'>
          <Progress.Indicator className={`h-full block ${fillColor(max > 0 ? value / max : 0)}`} />
        </Progress.Track>
      </Progress.Root>
      {showFraction && (
        <span class='text-text-muted text-xs font-mono'>
          {value} / {max}
        </span>
      )}
    </div>
  );
};
