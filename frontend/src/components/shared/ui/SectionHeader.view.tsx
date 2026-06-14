import { clsx } from 'clsx';
import type { ComponentChildren } from 'preact';

interface SectionHeaderProps {
  title: string;
  hint: string;
  action?: ComponentChildren;
  class?: string;
}

export const SectionHeader = ({ title, hint, action, class: classProp }: SectionHeaderProps) => (
  <div class={clsx('flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between', classProp)}>
    <div class='flex min-w-0 flex-col gap-1'>
      <h3 class='text-base font-semibold text-text-secondary uppercase tracking-wide'>{title}</h3>
      <p class='text-text-muted text-xs'>{hint}</p>
    </div>
    {action && <div class='shrink-0 sm:self-start'>{action}</div>}
  </div>
);
