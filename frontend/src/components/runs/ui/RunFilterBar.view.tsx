import type { Signal } from '@preact/signals';
import type { JSX } from 'preact';
import type { WorkRunStatus } from '../../../types/runs';

const STATUS_OPTIONS: { value: WorkRunStatus | ''; label: string }[] = [
  { value: '', label: 'All' },
  { value: 'pending', label: 'Pending' },
  { value: 'dispatched', label: 'Dispatched' },
  { value: 'running', label: 'Running' },
  { value: 'completed', label: 'Completed' },
  { value: 'failed', label: 'Failed' },
  { value: 'stalled', label: 'Stalled' }
];

interface RunFilterBarProps {
  statusFilter: Signal<WorkRunStatus | undefined>;
  onStatusFilter: (value: string) => void;
}

export const RunFilterBar = ({ statusFilter, onStatusFilter }: RunFilterBarProps): JSX.Element => (
  <select
    value={statusFilter.value ?? ''}
    onChange={(e) => onStatusFilter((e.target as HTMLSelectElement).value)}
    class='bg-bg-input border border-border-base text-text-primary text-sm px-3 py-2 focus:outline-none focus:border-border-focus'
  >
    {STATUS_OPTIONS.map((opt) => (
      <option key={opt.value} value={opt.value}>
        {opt.label}
      </option>
    ))}
  </select>
);
