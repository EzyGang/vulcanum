import type { Signal } from '@preact/signals';
import type { JSX } from 'preact';
import { Button } from './Button.view';

interface ConfirmDeleteProps {
  itemId: string;
  deletingId: Signal<string | null>;
  onConfirm: (id: string) => void;
  onDelete: (id: string) => void;
  onCancel: () => void;
  editActions?: JSX.Element;
}

export const ConfirmDelete = ({
  itemId,
  deletingId,
  onConfirm,
  onDelete,
  onCancel,
  editActions
}: ConfirmDeleteProps): JSX.Element => (
  <div class='flex items-center gap-3'>
    {deletingId.value === itemId ? (
      <div class='flex items-center gap-2'>
        <Button
          variant='danger'
          class='h-6 w-6 text-sm'
          aria-label='Confirm delete'
          onClick={(event) => {
            event.stopPropagation();
            onDelete(itemId);
          }}
        >
          ✓
        </Button>
        <Button
          variant='ghost'
          class='h-6 w-6 text-sm'
          aria-label='Cancel delete'
          onClick={(event) => {
            event.stopPropagation();
            onCancel();
          }}
        >
          ×
        </Button>
      </div>
    ) : (
      <div class='flex items-center gap-3'>
        {editActions}
        <Button
          variant='ghost-danger'
          class='h-6 w-6 text-base'
          aria-label='Delete'
          onClick={(event) => {
            event.stopPropagation();
            onConfirm(itemId);
          }}
        >
          ×
        </Button>
      </div>
    )}
  </div>
);
