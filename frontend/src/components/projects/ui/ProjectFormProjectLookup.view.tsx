import type { Signal } from '@preact/signals';
import type { JSX } from 'preact';
import { Button } from '../../shared/ui/Button.view';
import { Input } from '../../shared/ui/Input.view';
import { Label } from '../../shared/ui/Label.view';

interface ProjectFormProjectLookupProps {
  kaneoProjectId: Signal<string>;
  lookupProjectName: Signal<string>;
  lookupError: Signal<string | null>;
  columnsLoading: Signal<boolean>;
  isEdit: boolean;
  submitting: Signal<boolean>;
  actions: {
    onLookup: () => void;
    onProjectIdChange: (id: string) => void;
  };
}

export const ProjectFormProjectLookup = ({
  kaneoProjectId,
  lookupProjectName,
  lookupError,
  columnsLoading,
  isEdit,
  submitting,
  actions
}: ProjectFormProjectLookupProps): JSX.Element => (
  <div class='flex flex-col gap-2'>
    <Label for='field-kaneo-project-id'>Kaneo Project ID</Label>
    <div class='flex items-center gap-2'>
      <Input
        id='field-kaneo-project-id'
        type='text'
        value={kaneoProjectId.value}
        onInput={(e) => {
          actions.onProjectIdChange((e.target as HTMLInputElement).value);
        }}
        placeholder='e.g. k5s7dwb5f89anmaui2d814h9'
        disabled={isEdit || submitting.value}
        class='flex-1'
      />
      {!isEdit && (
        <Button
          variant='secondary'
          onClick={actions.onLookup}
          disabled={!kaneoProjectId.value || columnsLoading.value}
        >
          Look Up
        </Button>
      )}
    </div>
    {lookupError.value && <div class='text-error text-sm'>{lookupError.value}</div>}
    {lookupProjectName.value && (
      <div class='text-success text-sm'>Project: {lookupProjectName.value}</div>
    )}
  </div>
);
