import type { Signal } from '@preact/signals';
import { IconDeviceFloppy } from '@tabler/icons-react';
import type { JSX } from 'preact';
import type { SelectOption } from '../../../types/shared';
import type { TeamAgentBackend } from '../../../types/teams';
import { Button } from '../../shared/ui/Button.view';
import { CheckboxWithLabel } from '../../shared/ui/CheckboxWithLabel.view';
import { ErrorBanner } from '../../shared/ui/ErrorBanner.view';
import { Input } from '../../shared/ui/Input.view';
import { Label } from '../../shared/ui/Label.view';
import { TextArea } from '../../shared/ui/TextArea.view';
import { ModelSelectionFields } from './ModelSelectionFields.view';

export type TeamDefaultsSection = 'defaults' | 'models';

interface TeamDefaultsViewProps {
  section: TeamDefaultsSection;
  data: {
    promptTemplate: Signal<string>;
    promptTemplateInherited: Signal<boolean>;
    agentsMd: Signal<string>;
    primaryModelProviderKey: Signal<string>;
    primaryModelId: Signal<string>;
    smallModelProviderKey: Signal<string>;
    reviewPrimaryModelProviderKey: Signal<string>;
    reviewPrimaryModelId: Signal<string>;
    reviewSmallModelProviderKey: Signal<string>;
    reviewSmallModelId: Signal<string>;
    smallModelId: Signal<string>;
    reviewEnabled: Signal<boolean>;
    reviewMaxTurns: Signal<number>;
    reviewPromptTemplate: Signal<string>;
    reviewPromptTemplateInherited: Signal<boolean>;
    maxInProgressTasks: Signal<number>;
    agentBackend: Signal<TeamAgentBackend>;
    agentBackendItems: SelectOption[];
    connectedProviderItems: SelectOption[];
    primaryModelItems: SelectOption[];
    reviewPrimaryModelItems: SelectOption[];
    reviewSmallModelItems: SelectOption[];
    smallModelItems: SelectOption[];
  };
  status: {
    loading: boolean;
    saving: boolean;
    error: Signal<string | null>;
  };
  actions: {
    onPromptTemplateInput: (event: Event) => void;
    onResetPromptTemplate: () => void;
    onAgentsMdInput: (event: Event) => void;
    onPrimaryProviderChange: (value: string) => void;
    onPrimaryModelChange: (value: string) => void;
    onSmallProviderChange: (value: string) => void;
    onReviewPrimaryProviderChange: (value: string) => void;
    onReviewPrimaryModelChange: (value: string) => void;
    onReviewSmallProviderChange: (value: string) => void;
    onReviewSmallModelChange: (value: string) => void;
    onSmallModelChange: (value: string) => void;
    onAgentBackendChange: (value: string) => void;
    onReviewEnabledChange: (checked: boolean) => void;
    onReviewMaxTurnsInput: (event: Event) => void;
    onReviewPromptTemplateInput: (event: Event) => void;
    onResetReviewPromptTemplate: () => void;
    onMaxInProgressTasksInput: (event: Event) => void;
    onSubmit: (event: Event) => void;
  };
}

const SECTION_COPY: Record<
  TeamDefaultsSection,
  { title: string; description: string; save: string }
> = {
  defaults: {
    title: 'Agent defaults',
    description: 'Prompts, repository instructions, review behavior, and project concurrency.',
    save: 'Save agent defaults'
  },
  models: {
    title: 'Model selection',
    description: 'Default primary and small-model choices for project automation.',
    save: 'Save model selection'
  }
};

const SaveButton = ({ saving, label }: { saving: boolean; label: string }): JSX.Element => (
  <Button type='submit' variant='primary' disabled={saving} class='self-start'>
    <span class='inline-flex items-center gap-2'>
      <IconDeviceFloppy size={16} stroke={1.75} aria-hidden='true' />
      {saving ? 'Saving…' : label}
    </span>
  </Button>
);

const AgentDefaultFields = ({
  data,
  status,
  actions
}: Pick<TeamDefaultsViewProps, 'data' | 'status' | 'actions'>): JSX.Element => (
  <>
    <div class='flex flex-col gap-2'>
      <div class='flex items-center justify-between gap-3'>
        <Label for='team-default-prompt'>Prompt Template</Label>
        <Button
          type='button'
          variant='ghost'
          disabled={status.saving || data.promptTemplateInherited.value}
          onClick={actions.onResetPromptTemplate}
        >
          Reset to system default
        </Button>
      </div>
      <TextArea
        id='team-default-prompt'
        value={data.promptTemplate.value}
        onInput={actions.onPromptTemplateInput}
        rows={5}
        disabled={status.saving}
      />
    </div>

    <div class='flex flex-col gap-2'>
      <Label for='team-agents-md'>Agents.md</Label>
      <TextArea
        id='team-agents-md'
        value={data.agentsMd.value}
        onInput={actions.onAgentsMdInput}
        rows={6}
        disabled={status.saving}
      />
    </div>

    <div class='flex flex-col gap-4 border border-border-base bg-bg-card p-4'>
      <div class='flex flex-col gap-2'>
        <Label for='team-max-in-progress-tasks'>Project In-progress Limit</Label>
        <Input
          id='team-max-in-progress-tasks'
          type='number'
          min='1'
          value={data.maxInProgressTasks.value}
          onInput={actions.onMaxInProgressTasksInput}
          disabled={status.saving}
        />
      </div>
    </div>

    <div class='flex flex-col gap-4 border border-border-base bg-bg-card p-4'>
      <CheckboxWithLabel
        id='team-review-enabled'
        checked={data.reviewEnabled.value}
        onCheckedChange={actions.onReviewEnabledChange}
        disabled={status.saving}
      >
        Enable PR Review Automation
      </CheckboxWithLabel>
      <div class='grid grid-cols-1 gap-4 md:grid-cols-2'>
        <div class='flex flex-col gap-2'>
          <Label for='team-review-max-turns'>Review Follow-up Passes</Label>
          <Input
            id='team-review-max-turns'
            type='number'
            min='1'
            value={data.reviewMaxTurns.value}
            onInput={actions.onReviewMaxTurnsInput}
            disabled={status.saving}
          />
        </div>
      </div>
      <div class='flex flex-col gap-2'>
        <div class='flex items-center justify-between gap-3'>
          <Label for='team-review-prompt'>Review Prompt Template</Label>
          <Button
            type='button'
            variant='ghost'
            disabled={status.saving || data.reviewPromptTemplateInherited.value}
            onClick={actions.onResetReviewPromptTemplate}
          >
            Reset to system default
          </Button>
        </div>
        <TextArea
          id='team-review-prompt'
          value={data.reviewPromptTemplate.value}
          onInput={actions.onReviewPromptTemplateInput}
          rows={5}
          disabled={status.saving}
        />
      </div>
    </div>
  </>
);

export const TeamDefaultsView = ({
  section,
  data,
  status,
  actions
}: TeamDefaultsViewProps): JSX.Element => (
  <form onSubmit={actions.onSubmit} class='flex flex-col gap-5'>
    <div class='flex flex-col gap-1'>
      <h3 class='text-base font-semibold uppercase tracking-wide text-text-primary'>
        {SECTION_COPY[section].title}
      </h3>
      <p class='text-sm leading-relaxed text-text-muted'>{SECTION_COPY[section].description}</p>
    </div>
    {status.error.value && <ErrorBanner message={status.error.value} />}
    {status.loading && <div class='text-sm text-text-muted'>Loading team defaults...</div>}
    {!status.loading && (
      <>
        {section === 'models' ? (
          <ModelSelectionFields data={data} saving={status.saving} actions={actions} />
        ) : (
          <AgentDefaultFields data={data} status={status} actions={actions} />
        )}
        <SaveButton saving={status.saving} label={SECTION_COPY[section].save} />
      </>
    )}
  </form>
);
