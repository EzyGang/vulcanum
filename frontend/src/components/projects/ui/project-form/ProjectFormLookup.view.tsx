import type { JSX } from 'preact';
import { Button } from '../../../shared/ui/Button.view';
import { Input } from '../../../shared/ui/Input.view';
import { Label } from '../../../shared/ui/Label.view';
import { useProjectFormLookupContext } from '../../context/ProjectFormLookupContext';
import { useProjectFormMetaContext } from '../../context/ProjectFormMetaContext';

export const ProjectFormLookup = (): JSX.Element => {
  const m = useProjectFormMetaContext();
  const l = useProjectFormLookupContext();

  return (
    <div class='flex flex-col gap-2'>
      <Label for='field-kaneo-project-id'>Kaneo Project ID</Label>
      <div class='flex items-center gap-2'>
        <Input
          id='field-kaneo-project-id'
          type='text'
          value={l.externalProjectId.value}
          onInput={(e) => {
            l.onProjectIdChange((e.target as HTMLInputElement).value);
          }}
          placeholder='e.g. k5s7dwb5f89anmaui2d814h9'
          disabled={m.isEdit || m.submitting.value}
          class='flex-1'
        />
        {!m.isEdit && (
          <Button variant='secondary' onClick={l.onLookup} disabled={!l.externalProjectId.value}>
            Look Up
          </Button>
        )}
      </div>
      {l.lookupError.value && <div class='text-error text-sm'>{l.lookupError.value}</div>}
      {l.lookupProjectName.value && (
        <div class='text-success text-sm'>Project: {l.lookupProjectName.value}</div>
      )}
    </div>
  );
};
