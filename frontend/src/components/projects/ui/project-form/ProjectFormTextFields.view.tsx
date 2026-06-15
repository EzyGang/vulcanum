import type { JSX } from 'preact';
import { Checkbox } from '../../../shared/ui/Checkbox.view';
import { Label } from '../../../shared/ui/Label.view';
import { Select } from '../../../shared/ui/Select.view';
import { TextArea } from '../../../shared/ui/TextArea.view';
import { useProjectFormFieldsContext } from '../../context/ProjectFormFieldsContext';
import { useProjectFormMetaContext } from '../../context/ProjectFormMetaContext';

export const ProjectFormTextFields = (): JSX.Element => {
  const m = useProjectFormMetaContext();
  const f = useProjectFormFieldsContext();

  return (
    <>
      <div class='flex flex-col gap-2'>
        <Label for='field-repos'>Repositories</Label>
        <span class='text-text-muted text-xs'>
          Select all GitHub repositories this project needs.
        </span>
        {f.reposLoading && <span class='text-xs text-text-muted'>Loading repos...</span>}
        {!f.reposLoading && f.repoItems.length === 0 && (
          <span class='text-xs text-text-muted'>No accessible GitHub repositories found.</span>
        )}
        {f.repoItems.length > 0 && (
          <div
            id='field-repos'
            class='flex max-h-60 flex-col gap-2 overflow-auto border border-border-base bg-bg-card p-3'
          >
            {f.repoItems.map((repo) => (
              <div key={repo.fullName} class='flex items-center gap-2 text-sm text-text-secondary'>
                <Checkbox
                  checked={repo.checked}
                  onCheckedChange={repo.onCheckedChange}
                  disabled={m.submitting.value}
                />
                <span class='font-mono text-xs'>{repo.fullName}</span>
              </div>
            ))}
          </div>
        )}
      </div>

      <div class='flex flex-col gap-4 border border-border-base bg-bg-panel p-4'>
        <button
          type='button'
          class='flex items-center justify-between text-left text-sm font-semibold uppercase tracking-wide text-text-primary'
          onClick={f.onToggleOverrides}
          disabled={m.submitting.value}
        >
          <span>Project Overrides</span>
          <span class='text-text-muted'>{f.overridesToggleLabel}</span>
        </button>
        {!f.overridesOpen.value && (
          <span class='text-xs text-text-muted'>
            Prompt, model, and Agents.md settings inherit from the team.
          </span>
        )}
        {f.overridesOpen.value && (
          <>
            <div class='flex flex-col gap-2'>
              <div class='flex flex-col gap-1'>
                <Label for='field-prompt-template'>Prompt Template Override</Label>
                <span class='text-text-muted text-xs'>
                  Supports {'{{task_title}}'}, {'{{task_body}}'}, {'{{repo_url}}'},{' '}
                  {'{{repo_urls}}'}, and {'{{repo_layout}}'} variables.
                </span>
              </div>
              <TextArea
                id='field-prompt-template'
                value={f.promptTemplate.value}
                onInput={f.onPromptTemplateInput}
                disabled={m.submitting.value}
                rows={4}
              />
            </div>

            <div class='flex flex-col gap-2'>
              <Label for='field-primary-model-provider'>Primary Model Provider</Label>
              <Select
                id='field-primary-model-provider'
                value={f.primaryModelProviderKey.value}
                onValueChange={f.onPrimaryModelProviderChange}
                disabled={m.submitting.value}
                placeholder='Select a connected model provider...'
                items={f.connectedProviderItems}
              />
            </div>

            <div class='flex flex-col gap-2'>
              <Label for='field-primary-model'>Primary Model</Label>
              <Select
                id='field-primary-model'
                value={f.primaryModelId.value}
                onValueChange={f.onPrimaryModelChange}
                disabled={m.submitting.value || f.primaryModelItems.length === 0}
                placeholder='Select a model...'
                items={f.primaryModelItems}
              />
            </div>

            <div class='grid grid-cols-1 md:grid-cols-2 gap-4'>
              <div class='flex flex-col gap-2'>
                <Label for='field-small-model-provider'>Small Model Provider</Label>
                <Select
                  id='field-small-model-provider'
                  value={f.smallModelProviderKey.value}
                  onValueChange={f.onSmallModelProviderChange}
                  disabled={m.submitting.value}
                  placeholder='Optional provider...'
                  items={f.connectedProviderItems}
                />
              </div>
              <div class='flex flex-col gap-2'>
                <Label for='field-small-model'>Small Model</Label>
                <Select
                  id='field-small-model'
                  value={f.smallModelId.value}
                  onValueChange={f.onSmallModelChange}
                  disabled={m.submitting.value || f.smallModelItems.length === 0}
                  placeholder='Optional model...'
                  items={f.smallModelItems}
                />
              </div>
            </div>

            <div class='flex flex-col gap-2'>
              <Label for='field-agents-md'>Agents.md Override</Label>
              <span class='text-text-muted text-xs'>
                Global agent guide for this project. Does not overwrite any per-repo AGENTS.md file.
              </span>
              <TextArea
                id='field-agents-md'
                value={f.agentsMd.value}
                onInput={f.onAgentsMdInput}
                disabled={m.submitting.value}
                rows={6}
              />
            </div>
          </>
        )}
      </div>
    </>
  );
};
