import { IconUserCheck } from '@tabler/icons-react';
import type { JSX } from 'preact';
import { Button } from '../../shared/ui/Button.view';

interface GitHubReviewIdentityPanelProps {
  statusText: string;
  actionLabel: string;
  actionPending: boolean;
  onAction: () => void;
}

export const GitHubReviewIdentityPanel = ({
  statusText,
  actionLabel,
  actionPending,
  onAction
}: GitHubReviewIdentityPanelProps): JSX.Element => (
  <div class='flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3 border border-border-base bg-bg-panel p-4'>
    <div class='flex flex-col gap-1'>
      <span class='text-text-primary text-sm font-medium'>PR review identity</span>
      <span class='text-text-muted text-xs'>{statusText}</span>
    </div>
    <Button variant='secondary' onClick={onAction} disabled={actionPending}>
      <span class='inline-flex items-center gap-2'>
        <IconUserCheck size={16} stroke={1.75} aria-hidden='true' />
        {actionLabel}
      </span>
    </Button>
  </div>
);
