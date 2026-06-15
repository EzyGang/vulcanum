import type { JSX } from 'preact';
import { Button } from '../../shared/ui/Button.view';
import { ErrorBanner } from '../../shared/ui/ErrorBanner.view';
import { Input } from '../../shared/ui/Input.view';
import { Label } from '../../shared/ui/Label.view';
import { TextArea } from '../../shared/ui/TextArea.view';

interface TeamDefaultsViewProps {
  data: {
    promptTemplate: string;
    agentsMd: string;
    primaryModelProviderKey: string;
    primaryModelId: string;
    smallModelProviderKey: string;
    smallModelId: string;
  };
  status: {
    loading: boolean;
    saving: boolean;
    error: string | null;
  };
  actions: {
    onPromptTemplateInput: (event: Event) => void;
    onAgentsMdInput: (event: Event) => void;
    onPrimaryProviderInput: (event: Event) => void;
    onPrimaryModelInput: (event: Event) => void;
    onSmallProviderInput: (event: Event) => void;
    onSmallModelInput: (event: Event) => void;
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
    {status.error && <ErrorBanner message={status.error} />}
    {status.loading && <div class='text-sm text-text-muted'>Loading team defaults...</div>}
    {!status.loading && (
      <>
        <div class='flex flex-col gap-2'>
          <Label for='team-default-prompt'>Prompt Template</Label>
          <TextArea
            id='team-default-prompt'
            value={data.promptTemplate}
            onInput={actions.onPromptTemplateInput}
            rows={5}
            disabled={status.saving}
          />
        </div>
        <div class='grid grid-cols-1 gap-4 md:grid-cols-2'>
          <div class='flex flex-col gap-2'>
            <Label for='team-primary-provider'>Primary Model Provider</Label>
            <Input
              id='team-primary-provider'
              value={data.primaryModelProviderKey}
              onInput={actions.onPrimaryProviderInput}
              disabled={status.saving}
            />
          </div>
          <div class='flex flex-col gap-2'>
            <Label for='team-primary-model'>Primary Model</Label>
            <Input
              id='team-primary-model'
              value={data.primaryModelId}
              onInput={actions.onPrimaryModelInput}
              disabled={status.saving}
            />
          </div>
          <div class='flex flex-col gap-2'>
            <Label for='team-small-provider'>Small Model Provider</Label>
            <Input
              id='team-small-provider'
              value={data.smallModelProviderKey}
              onInput={actions.onSmallProviderInput}
              disabled={status.saving}
            />
          </div>
          <div class='flex flex-col gap-2'>
            <Label for='team-small-model'>Small Model</Label>
            <Input
              id='team-small-model'
              value={data.smallModelId}
              onInput={actions.onSmallModelInput}
              disabled={status.saving}
            />
          </div>
        </div>
        <div class='flex flex-col gap-2'>
          <Label for='team-agents-md'>Agents.md</Label>
          <TextArea
            id='team-agents-md'
            value={data.agentsMd}
            onInput={actions.onAgentsMdInput}
            rows={6}
            disabled={status.saving}
          />
        </div>
        <Button type='submit' variant='primary' disabled={status.saving}>
          {status.saving ? 'Saving...' : 'Save Team Defaults'}
        </Button>
      </>
    )}
  </form>
);
