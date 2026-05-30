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
        <span class='text-text-muted text-xs'>Confirm?</span>
        <Button variant='danger' onClick={() => onDelete(itemId)}>
          Delete
        </Button>
        <Button variant='ghost' onClick={onCancel}>
          Cancel
        </Button>
      </div>
    ) : (
      <div class='flex items-center gap-3'>
        {editActions}
        <Button variant='ghost-danger' onClick={() => onConfirm(itemId)}>
          Delete
        </Button>
      </div>
    )}
  </div>
);
