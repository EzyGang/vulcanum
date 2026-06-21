import {
  IconCheck,
  IconCircleX,
  IconMenu2,
  IconPlayerStop,
  IconTrash,
  IconX
} from '@tabler/icons-react';
import type { JSX } from 'preact';
import { ActionIconButton } from '../../../shared/ui/ActionIconButton.view';
import { Button } from '../../../shared/ui/Button.view';

interface RunActionsProps {
  runId: string;
  cancellable: boolean;
  deleting: boolean;
  onFailRun: (id: string) => void;
  onCancelRun: (id: string) => void;
  onConfirmDelete: (id: string) => void;
  onDelete: (id: string) => void;
  onCancelDelete: () => void;
}

export const RunActions = ({
  runId,
  cancellable,
  deleting,
  onFailRun,
  onCancelRun,
  onConfirmDelete,
  onDelete,
  onCancelDelete
}: RunActionsProps): JSX.Element => (
  <>
    <div class='hidden max-w-21 flex-wrap gap-1 lg:inline-flex'>
      {cancellable && (
        <ActionIconButton label='Cancel run' variant='critical' onClick={() => onCancelRun(runId)}>
          <IconPlayerStop size={18} stroke={1.75} aria-hidden='true' />
        </ActionIconButton>
      )}
      {cancellable && (
        <ActionIconButton label='Mark run failed' onClick={() => onFailRun(runId)}>
          <IconCircleX size={18} stroke={1.75} aria-hidden='true' />
        </ActionIconButton>
      )}
      {deleting ? (
        <>
          <ActionIconButton label='Confirm delete' onClick={() => onDelete(runId)}>
            <IconCheck size={18} stroke={1.75} aria-hidden='true' />
          </ActionIconButton>
          <ActionIconButton label='Cancel delete' onClick={onCancelDelete}>
            <IconX size={18} stroke={1.75} aria-hidden='true' />
          </ActionIconButton>
        </>
      ) : (
        <ActionIconButton label='Delete' onClick={() => onConfirmDelete(runId)}>
          <IconTrash size={18} stroke={1.75} aria-hidden='true' />
        </ActionIconButton>
      )}
    </div>
    <details class='relative lg:hidden'>
      <summary class='flex h-10 w-10 cursor-pointer list-none items-center justify-center border border-transparent text-text-muted hover:border-border-base hover:bg-bg-hover hover:text-text-primary [&::-webkit-details-marker]:hidden'>
        <span class='sr-only'>Open run actions</span>
        <IconMenu2 size={20} stroke={1.75} aria-hidden='true' />
      </summary>
      <div class='absolute right-0 z-10 mt-1 flex min-w-40 flex-col border border-border-base bg-bg-card p-1'>
        {cancellable && (
          <Button
            type='button'
            variant='danger'
            class='justify-start px-3 py-2 text-left text-sm normal-case tracking-normal hover:bg-bg-hover'
            onClick={() => onCancelRun(runId)}
          >
            Cancel run
          </Button>
        )}
        {cancellable && (
          <Button
            type='button'
            variant='ghost'
            class='justify-start px-3 py-2 text-left text-sm normal-case tracking-normal hover:bg-bg-hover'
            onClick={() => onFailRun(runId)}
          >
            Mark failed
          </Button>
        )}
        {deleting ? (
          <>
            <Button
              type='button'
              variant='ghost'
              class='justify-start px-3 py-2 text-left text-sm normal-case tracking-normal hover:bg-bg-hover'
              onClick={() => onDelete(runId)}
            >
              Confirm delete
            </Button>
            <Button
              type='button'
              variant='ghost'
              class='justify-start px-3 py-2 text-left text-sm normal-case tracking-normal hover:bg-bg-hover'
              onClick={onCancelDelete}
            >
              Cancel delete
            </Button>
          </>
        ) : (
          <Button
            type='button'
            variant='ghost'
            class='justify-start px-3 py-2 text-left text-sm normal-case tracking-normal hover:bg-bg-hover'
            onClick={() => onConfirmDelete(runId)}
          >
            Delete
          </Button>
        )}
      </div>
    </details>
  </>
);
