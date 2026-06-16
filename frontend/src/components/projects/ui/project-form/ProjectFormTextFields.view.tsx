import type { JSX } from 'preact';
import { Checkbox } from '../../../shared/ui/Checkbox.view';
import { Label } from '../../../shared/ui/Label.view';
import { Select } from '../../../shared/ui/Select.view';
import { TextArea } from '../../../shared/ui/TextArea.view';
import { useProjectFormFieldsContext } from '../../context/ProjectFormFieldsContext';
import { useProjectFormMetaContext } from '../../context/ProjectFormMetaContext';
import { OverrideResetButton } from './OverrideResetButton.view';
import { ProjectFormColumnSelect } from './ProjectFormColumnSelect.view';

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
            {f.hasOverrides
              ? 'This project has override values. Expand to view or reset them.'
              : 'Prompt, model, and Agents.md settings inherit from the team.'}
          </span>
        )}
        {f.overridesOpen.value && (
          <>
            <div class='flex flex-col gap-2'>
              <div class='flex flex-col gap-1'>
                <div class='flex items-center justify-between gap-2'>
                  <Label for='field-prompt-template'>Prompt Template Override</Label>
                  <OverrideResetButton
                    label='Use team default prompt template'
                    disabled={m.submitting.value || !f.promptTemplateOverride.value}
                    onClick={f.onResetPromptTemplateOverride}
                  />
                </div>
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
              <div class='flex items-center justify-between gap-2'>
                <Label for='field-primary-model-provider'>Primary Model Provider</Label>
                <OverrideResetButton
                  label='Use team default primary model provider'
                  disabled={m.submitting.value || !f.primaryModelProviderOverride.value}
                  onClick={f.onResetPrimaryModelProviderOverride}
                />
              </div>
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
              <div class='flex items-center justify-between gap-2'>
                <Label for='field-primary-model'>Primary Model</Label>
                <OverrideResetButton
                  label='Use team default primary model'
                  disabled={m.submitting.value || !f.primaryModelIdOverride.value}
                  onClick={f.onResetPrimaryModelOverride}
                />
              </div>
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
                <div class='flex items-center justify-between gap-2'>
                  <Label for='field-small-model-provider'>Small Model Provider</Label>
                  <OverrideResetButton
                    label='Use team default small model provider'
                    disabled={m.submitting.value || !f.smallModelProviderOverride.value}
                    onClick={f.onResetSmallModelProviderOverride}
                  />
                </div>
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
                <div class='flex items-center justify-between gap-2'>
                  <Label for='field-small-model'>Small Model</Label>
                  <OverrideResetButton
                    label='Use team default small model'
                    disabled={m.submitting.value || !f.smallModelIdOverride.value}
                    onClick={f.onResetSmallModelOverride}
                  />
                </div>
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
              <div class='flex items-center justify-between gap-2'>
                <Label for='field-agents-md'>Agents.md Override</Label>
                <OverrideResetButton
                  label='Use team default Agents.md'
                  disabled={m.submitting.value || !f.agentsMdOverride.value}
                  onClick={f.onResetAgentsMdOverride}
                />
              </div>
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

            <div class='flex flex-col gap-4 border border-border-base bg-bg-card p-3'>
              <div class='flex items-center justify-between gap-2'>
                <span class='text-xs font-semibold uppercase tracking-wide text-text-primary'>
                  PR Review Automation Overrides
                </span>
                <OverrideResetButton
                  label='Reset review automation overrides'
                  disabled={
                    m.submitting.value ||
                    (!f.reviewEnabledOverride.value &&
                      !f.reviewPickupColumnOverride.value &&
                      !f.reviewMaxTurnsOverride.value &&
                      !f.reviewPromptTemplateOverride.value)
                  }
                  onClick={() => {
                    f.onResetReviewEnabledOverride();
                    f.onResetReviewPickupColumnOverride();
                    f.onResetReviewMaxTurnsOverride();
                    f.onResetReviewPromptTemplateOverride();
                  }}
                />
              </div>
              <label for='field-review-enabled' class='flex items-center gap-2 cursor-pointer'>
                <input
                  id='field-review-enabled'
                  type='checkbox'
                  checked={f.reviewEnabled.value}
                  onChange={(event) =>
                    f.onReviewEnabledChange((event.target as HTMLInputElement).checked)
                  }
                  disabled={m.submitting.value}
                />
                <span class='text-sm text-text-primary'>Override review automation enabled</span>
              </label>
              <ProjectFormColumnSelect
                id='field-review-pickup-column'
                label='Review Pickup Column Override'
                value={f.reviewPickupColumn.value}
                columns={f.columns.value}
                columnsLoading={f.columnsLoading.value}
                disabled={m.submitting.value}
                placeholderText='Select review pickup column'
                onChange={f.onReviewPickupColumnChange}
              />
              <div class='flex flex-col gap-2'>
                <Label for='field-review-max-turns'>Review Max Turns Override</Label>
                <input
                  id='field-review-max-turns'
                  class='border border-border-base bg-bg-card px-3 py-2 text-sm text-text-primary'
                  type='number'
                  min='1'
                  value={f.reviewMaxTurns.value}
                  onInput={f.onReviewMaxTurnsInput}
                  disabled={m.submitting.value}
                />
              </div>
              <div class='flex flex-col gap-2'>
                <Label for='field-review-prompt-template'>Review Prompt Template Override</Label>
                <TextArea
                  id='field-review-prompt-template'
                  value={f.reviewPromptTemplate.value}
                  onInput={f.onReviewPromptTemplateInput}
                  disabled={m.submitting.value}
                  rows={4}
                />
              </div>
            </div>
          </>
        )}
      </div>
    </>
  );
};
