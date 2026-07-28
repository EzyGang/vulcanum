import type { Signal } from '@preact/signals';
import type { JSX } from 'preact';
import type { SelectOption } from '../../../types/shared';
import type { TeamAgentBackend } from '../../../types/teams';
import { Label } from '../../shared/ui/Label.view';
import { Select } from '../../shared/ui/Select.view';

interface ModelSelectionFieldsProps {
  data: {
    agentBackend: Signal<TeamAgentBackend>;
    agentBackendItems: SelectOption[];
    connectedProviderItems: SelectOption[];
    primaryModelProviderKey: Signal<string>;
    primaryModelId: Signal<string>;
    primaryModelItems: SelectOption[];
    smallModelProviderKey: Signal<string>;
    smallModelId: Signal<string>;
    smallModelItems: SelectOption[];
    reviewPrimaryModelProviderKey: Signal<string>;
    reviewPrimaryModelId: Signal<string>;
    reviewPrimaryModelItems: SelectOption[];
    reviewSmallModelProviderKey: Signal<string>;
    reviewSmallModelId: Signal<string>;
    reviewSmallModelItems: SelectOption[];
  };
  saving: boolean;
  actions: {
    onAgentBackendChange: (value: string) => void;
    onPrimaryProviderChange: (value: string) => void;
    onPrimaryModelChange: (value: string) => void;
    onSmallProviderChange: (value: string) => void;
    onSmallModelChange: (value: string) => void;
    onReviewPrimaryProviderChange: (value: string) => void;
    onReviewPrimaryModelChange: (value: string) => void;
    onReviewSmallProviderChange: (value: string) => void;
    onReviewSmallModelChange: (value: string) => void;
  };
}

export const ModelSelectionFields = ({
  data,
  saving,
  actions
}: ModelSelectionFieldsProps): JSX.Element => (
  <div class='grid grid-cols-1 gap-4 xl:grid-cols-2'>
    <RuntimeControls
      title='Implementation runtime'
      description='Used for implementation work runs.'
      primary={{
        providerId: 'team-primary-provider',
        modelId: 'team-primary-model',
        provider: data.primaryModelProviderKey,
        model: data.primaryModelId,
        modelItems: data.primaryModelItems,
        onProviderChange: actions.onPrimaryProviderChange,
        onModelChange: actions.onPrimaryModelChange
      }}
      small={{
        providerId: 'team-small-provider',
        modelId: 'team-small-model',
        provider: data.smallModelProviderKey,
        model: data.smallModelId,
        modelItems: data.smallModelItems,
        onProviderChange: actions.onSmallProviderChange,
        onModelChange: actions.onSmallModelChange
      }}
      connectedProviderItems={data.connectedProviderItems}
      saving={saving}
    >
      <Label for='team-agent-backend'>Agent Backend</Label>
      <Select
        id='team-agent-backend'
        value={data.agentBackend.value}
        onValueChange={actions.onAgentBackendChange}
        disabled={saving}
        placeholder='Select an agent backend...'
        items={data.agentBackendItems}
      />
    </RuntimeControls>
    <RuntimeControls
      title='Review runtime'
      description='Optional overrides for pull-request reviews. Unset pairs use the corresponding implementation pair.'
      primary={{
        providerId: 'team-review-primary-provider',
        modelId: 'team-review-primary-model',
        provider: data.reviewPrimaryModelProviderKey,
        model: data.reviewPrimaryModelId,
        modelItems: data.reviewPrimaryModelItems,
        onProviderChange: actions.onReviewPrimaryProviderChange,
        onModelChange: actions.onReviewPrimaryModelChange
      }}
      small={{
        providerId: 'team-review-small-provider',
        modelId: 'team-review-small-model',
        provider: data.reviewSmallModelProviderKey,
        model: data.reviewSmallModelId,
        modelItems: data.reviewSmallModelItems,
        onProviderChange: actions.onReviewSmallProviderChange,
        onModelChange: actions.onReviewSmallModelChange
      }}
      connectedProviderItems={data.connectedProviderItems}
      saving={saving}
    />
  </div>
);

interface PairProps {
  providerId: string;
  modelId: string;
  provider: Signal<string>;
  model: Signal<string>;
  modelItems: SelectOption[];
  onProviderChange: (value: string) => void;
  onModelChange: (value: string) => void;
}

interface RuntimeControlsProps {
  title: string;
  description: string;
  primary: PairProps;
  small: PairProps;
  connectedProviderItems: SelectOption[];
  saving: boolean;
  children?: JSX.Element | JSX.Element[];
}

const RuntimeControls = ({
  title,
  description,
  primary,
  small,
  connectedProviderItems,
  saving,
  children
}: RuntimeControlsProps): JSX.Element => (
  <section class='flex flex-col gap-4 border border-border-base bg-bg-card p-4'>
    <div class='flex flex-col gap-1'>
      <span class='text-xs font-medium uppercase tracking-wider text-accent'>{title}</span>
      <p class='text-xs leading-relaxed text-text-muted'>{description}</p>
    </div>
    {children ? <div class='flex flex-col gap-2'>{children}</div> : null}
    <ModelPair
      label='Primary'
      pair={primary}
      connectedProviderItems={connectedProviderItems}
      saving={saving}
    />
    <ModelPair
      label='Small model'
      pair={small}
      connectedProviderItems={connectedProviderItems}
      saving={saving}
    />
  </section>
);

const ModelPair = ({
  label,
  pair,
  connectedProviderItems,
  saving
}: {
  label: string;
  pair: PairProps;
  connectedProviderItems: SelectOption[];
  saving: boolean;
}): JSX.Element => (
  <div class='flex flex-col gap-2'>
    <Label for={pair.providerId}>{`${label} provider`}</Label>
    <Select
      id={pair.providerId}
      value={pair.provider.value}
      onValueChange={pair.onProviderChange}
      disabled={saving}
      placeholder='Optional provider...'
      items={connectedProviderItems}
    />
    <Label for={pair.modelId}>{`${label} model`}</Label>
    <Select
      id={pair.modelId}
      value={pair.model.value}
      onValueChange={pair.onModelChange}
      disabled={saving || pair.modelItems.length === 0}
      placeholder='Optional model...'
      items={pair.modelItems}
    />
  </div>
);
