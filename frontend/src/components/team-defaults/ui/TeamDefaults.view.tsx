import type { Signal } from '@preact/signals';
import type { JSX } from 'preact';
import type { SelectOption } from '../../../types/shared';
import { Button } from '../../shared/ui/Button.view';
import { CheckboxWithLabel } from '../../shared/ui/CheckboxWithLabel.view';
import { ErrorBanner } from '../../shared/ui/ErrorBanner.view';
import { Input } from '../../shared/ui/Input.view';
import { Label } from '../../shared/ui/Label.view';
import { Select } from '../../shared/ui/Select.view';
import { TextArea } from '../../shared/ui/TextArea.view';

interface TeamDefaultsViewProps {
  data: {
    promptTemplate: Signal<string>;
    agentsMd: Signal<string>;
    primaryModelProviderKey: Signal<string>;
    primaryModelId: Signal<string>;
    smallModelProviderKey: Signal<string>;
    smallModelId: Signal<string>;
    reviewEnabled: Signal<boolean>;
    reviewPickupColumn: Signal<string>;
    reviewMaxTurns: Signal<number>;
    reviewPromptTemplate: Signal<string>;
    connectedProviderItems: SelectOption[];
    primaryModelItems: SelectOption[];
    smallModelItems: SelectOption[];
  };
  status: {
    loading: boolean;
    saving: boolean;
    error: Signal<string | null>;
  };
  actions: {
    onPromptTemplateInput: (event: Event) => void;
    onAgentsMdInput: (event: Event) => void;
    onPrimaryProviderChange: (value: string) => void;
    onPrimaryModelChange: (value: string) => void;
    onSmallProviderChange: (value: string) => void;
    onSmallModelChange: (value: string) => void;
    onReviewEnabledChange: (checked: boolean) => void;
    onReviewPickupColumnInput: (event: Event) => void;
    onReviewMaxTurnsInput: (event: Event) => void;
    onReviewPromptTemplateInput: (event: Event) => void;
    onSubmit: (event: Event) => void;
  };
}

export const TeamDefaultsView = ({ data, status, actions }: TeamDefaultsViewProps): JSX.Element => (
  <form
    onSubmit={actions.onSubmit}
    class='flex flex-col gap-4 border border-border-base bg-bg-card p-5'
  >
    <div class='flex flex-col gap-1'>
      <h3 class='text-sm font-semibold uppercase tracking-wide text-text-primary'>Team Defaults</h3>
      <p class='text-xs text-text-muted'>
        Applied to every project unless a project override is enabled.
      </p>
    </div>
    {status.error.value && <ErrorBanner message={status.error.value} />}
    {status.loading && <div class='text-sm text-text-muted'>Loading team defaults...</div>}
    {!status.loading && (
      <>
        <div class='flex flex-col gap-2'>
          <Label for='team-default-prompt'>Prompt Template</Label>
          <TextArea
            id='team-default-prompt'
            value={data.promptTemplate.value}
            onInput={actions.onPromptTemplateInput}
            rows={5}
            disabled={status.saving}
          />
        </div>
        <div class='grid grid-cols-1 gap-4 md:grid-cols-2'>
          <div class='flex flex-col gap-2'>
            <Label for='team-primary-provider'>Primary Model Provider</Label>
            <Select
              id='team-primary-provider'
              value={data.primaryModelProviderKey.value}
              onValueChange={actions.onPrimaryProviderChange}
              disabled={status.saving}
              placeholder='Select a connected model provider...'
              items={data.connectedProviderItems}
            />
          </div>
          <div class='flex flex-col gap-2'>
            <Label for='team-primary-model'>Primary Model</Label>
            <Select
              id='team-primary-model'
              value={data.primaryModelId.value}
              onValueChange={actions.onPrimaryModelChange}
              disabled={status.saving || data.primaryModelItems.length === 0}
              placeholder='Select a model...'
              items={data.primaryModelItems}
            />
          </div>
          <div class='flex flex-col gap-2'>
            <Label for='team-small-provider'>Small Model Provider</Label>
            <Select
              id='team-small-provider'
              value={data.smallModelProviderKey.value}
              onValueChange={actions.onSmallProviderChange}
              disabled={status.saving}
              placeholder='Optional provider...'
              items={data.connectedProviderItems}
            />
          </div>
          <div class='flex flex-col gap-2'>
            <Label for='team-small-model'>Small Model</Label>
            <Select
              id='team-small-model'
              value={data.smallModelId.value}
              onValueChange={actions.onSmallModelChange}
              disabled={status.saving || data.smallModelItems.length === 0}
              placeholder='Optional model...'
              items={data.smallModelItems}
            />
          </div>
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
        <div class='flex flex-col gap-4 border border-border-base bg-bg-panel p-4'>
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
              <Label for='team-review-pickup-column'>Review Pickup Column</Label>
              <Input
                id='team-review-pickup-column'
                value={data.reviewPickupColumn.value}
                onInput={actions.onReviewPickupColumnInput}
                disabled={status.saving}
              />
            </div>
            <div class='flex flex-col gap-2'>
              <Label for='team-review-max-turns'>Review Max Turns</Label>
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
            <Label for='team-review-prompt'>Review Prompt Template</Label>
            <TextArea
              id='team-review-prompt'
              value={data.reviewPromptTemplate.value}
              onInput={actions.onReviewPromptTemplateInput}
              rows={5}
              disabled={status.saving}
            />
          </div>
        </div>
        <Button type='submit' variant='primary' disabled={status.saving}>
          {status.saving ? 'Saving...' : 'Save Team Defaults'}
        </Button>
      </>
    )}
  </form>
);
