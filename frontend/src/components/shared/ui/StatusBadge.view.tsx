import type { JSX } from 'preact';

const STATUS_COLORS: Record<string, string> = {
  pending: 'text-text-muted bg-bg-hover border-border-base',
  dispatched: 'text-accent-secondary bg-warning-bg border-warning-border',
  running: 'text-accent bg-success-bg border-success-border',
  completed: 'text-success bg-success-bg border-success-border',
  failed: 'text-error bg-error-bg border-error-border',
  stalled: 'text-warning bg-warning-bg border-warning-border',
  idle: 'text-success bg-success-bg border-success-border',
  busy: 'text-warning bg-warning-bg border-warning-border',
  disconnected: 'text-error bg-error-bg border-error-border'
};

export const StatusBadge = ({ status }: { status: string }): JSX.Element => (
  <span
    class={`text-xs uppercase tracking-wider px-2 py-0.5 border ${STATUS_COLORS[status] ?? 'text-text-muted bg-bg-hover border-border-base'}`}
  >
    {status}
  </span>
);
