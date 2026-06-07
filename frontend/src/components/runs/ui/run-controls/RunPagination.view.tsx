import type { Signal } from '@preact/signals';
import type { JSX } from 'preact';
import { Button } from '../../../shared/ui/Button.view';

interface RunPaginationProps {
  page: Signal<number>;
  hasPrevPage: boolean;
  hasNextPage: boolean;
  loading: boolean;
  onPrev: () => void;
  onNext: () => void;
}

export const RunPagination = ({
  page,
  hasPrevPage,
  hasNextPage,
  loading,
  onPrev,
  onNext
}: RunPaginationProps): JSX.Element => (
  <div class='flex items-center justify-between pt-4'>
    <Button variant='ghost' onClick={onPrev} disabled={!hasPrevPage || loading}>
      Previous
    </Button>
    <span class='text-text-muted text-sm'>Page {page.value + 1}</span>
    <Button variant='ghost' onClick={onNext} disabled={!hasNextPage || loading}>
      Next
    </Button>
  </div>
);
