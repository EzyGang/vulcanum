import type { JSX } from 'preact';

interface EmptyStateProps {
  title: string;
  description?: string;
  action?: JSX.Element;
}

export const EmptyState = ({ title, description, action }: EmptyStateProps): JSX.Element => (
  <div class='flex flex-col items-center gap-4 bg-bg-card border border-border-base p-12'>
    <p class='text-text-muted text-sm'>{title}</p>
    {description && <p class='text-text-muted text-xs'>{description}</p>}
    {action}
  </div>
);
