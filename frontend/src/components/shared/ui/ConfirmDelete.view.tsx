import type { Signal } from '@preact/signals';
import { IconCheck, IconTrash, IconX } from '@tabler/icons-react';
import type { JSX } from 'preact';
import { ActionIconButton } from './ActionIconButton.view';

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
  <div class='flex items-center gap-1'>
    {deletingId.value === itemId ? (
      <div class='flex items-center gap-1'>
        <ActionIconButton label='Confirm delete' variant='danger' onClick={() => onDelete(itemId)}>
          <IconCheck size={16} stroke={1.75} aria-hidden='true' />
        </ActionIconButton>
        <ActionIconButton label='Cancel delete' onClick={onCancel}>
          <IconX size={16} stroke={1.75} aria-hidden='true' />
        </ActionIconButton>
      </div>
    ) : (
      <div class='flex items-center gap-1'>
        {editActions}
        <ActionIconButton label='Delete' variant='danger' onClick={() => onConfirm(itemId)}>
          <IconTrash size={16} stroke={1.75} aria-hidden='true' />
        </ActionIconButton>
      </div>
    )}
  </div>
);
