import type { JSX } from 'preact';
import type { WorkRunStatus } from '../../../../types/runs';
import { Select } from '../../../shared/ui/Select.view';

const STATUS_OPTIONS: { value: string; label: string }[] = [
  { value: '', label: 'All' },
  { value: 'pending', label: 'Pending' },
  { value: 'dispatched', label: 'Dispatched' },
  { value: 'running', label: 'Running' },
  { value: 'completed', label: 'Completed' },
  { value: 'failed', label: 'Failed' },
  { value: 'stalled', label: 'Stalled' }
];

interface RunFilterBarProps {
  statusFilter: WorkRunStatus | undefined;
  onStatusFilter: (value: string) => void;
}

export const RunFilterBar = ({ statusFilter, onStatusFilter }: RunFilterBarProps): JSX.Element => (
  <Select
    value={statusFilter ?? ''}
    onValueChange={onStatusFilter}
    items={STATUS_OPTIONS}
    class='w-auto min-w-36'
  />
);
