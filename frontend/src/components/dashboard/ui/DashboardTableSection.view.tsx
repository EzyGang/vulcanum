import type { JSX } from 'preact';
import { Button } from '../../shared/ui/Button.view';

interface DashboardTableSectionProps {
  title: string;
  onViewAll: () => void;
  emptyMessage: string;
  isEmpty: boolean;
  children: JSX.Element;
}

export const DashboardTableSection = ({
  title,
  onViewAll,
  emptyMessage,
  isEmpty,
  children
}: DashboardTableSectionProps): JSX.Element => (
  <section class='flex flex-col gap-4'>
    <div class='flex items-center justify-between'>
      <h3 class='text-md font-semibold text-text-primary uppercase tracking-wide'>{title}</h3>
      <Button variant='ghost' onClick={onViewAll}>
        View all →
      </Button>
    </div>
    {isEmpty ? (
      <p class='text-text-muted text-sm'>{emptyMessage}</p>
    ) : (
      <div class='max-h-56 overflow-y-auto'>{children}</div>
    )}
  </section>
);
