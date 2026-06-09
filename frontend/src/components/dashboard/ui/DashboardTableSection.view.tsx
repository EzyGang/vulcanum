import type { JSX } from 'preact';

interface DashboardTableSectionProps {
  title: string;
  emptyMessage: string;
  isEmpty: boolean;
  children: JSX.Element;
}

export const DashboardTableSection = ({
  title,
  emptyMessage,
  isEmpty,
  children
}: DashboardTableSectionProps): JSX.Element => (
  <section class='flex flex-col gap-4'>
    <h3 class='text-base font-semibold text-text-primary uppercase tracking-wide'>{title}</h3>
    {isEmpty ? (
      <p class='text-text-muted text-sm'>{emptyMessage}</p>
    ) : (
      <div class='max-h-56 overflow-y-auto'>{children}</div>
    )}
  </section>
);
