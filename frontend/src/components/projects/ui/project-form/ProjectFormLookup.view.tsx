import type { JSX } from 'preact';
import { Button } from '../../../shared/ui/Button.view';
import { Input } from '../../../shared/ui/Input.view';
import { Label } from '../../../shared/ui/Label.view';
import { useProjectFormContext } from '../../context/ProjectFormContext';

export const ProjectFormLookup = (): JSX.Element => {
  const { data: d, status, actions: a } = useProjectFormContext();

  return (
    <div class='flex flex-col gap-2'>
      <Label for='field-kaneo-project-id'>Kaneo Project ID</Label>
      <div class='flex items-center gap-2'>
        <Input
          id='field-kaneo-project-id'
          type='text'
          value={d.externalProjectId.value}
          onInput={(e) => {
            a.onProjectIdChange((e.target as HTMLInputElement).value);
          }}
          placeholder='e.g. k5s7dwb5f89anmaui2d814h9'
          disabled={d.isEdit || status.submitting.value}
          class='flex-1'
        />
        {!d.isEdit && (
          <Button
            variant='secondary'
            onClick={a.onLookup}
            disabled={!d.externalProjectId.value || d.columnsLoading.value}
          >
            Look Up
          </Button>
        )}
      </div>
      {d.lookupError.value && <div class='text-error text-sm'>{d.lookupError.value}</div>}
      {d.lookupProjectName.value && (
        <div class='text-success text-sm'>Project: {d.lookupProjectName.value}</div>
      )}
    </div>
  );
};
