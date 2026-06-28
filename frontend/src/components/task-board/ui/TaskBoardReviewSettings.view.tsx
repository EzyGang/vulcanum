import type { JSX } from 'preact';
import { Input } from '../../shared/ui/Input.view';
import { Label } from '../../shared/ui/Label.view';
import { Select } from '../../shared/ui/Select.view';
import { TextArea } from '../../shared/ui/TextArea.view';
import type { TaskBoardActions, TaskBoardFormState, TaskBoardReviewSettingsData } from '../types';
import { TaskBoardSettingsSection } from './TaskBoardSettingsSection.view';

interface TaskBoardReviewSettingsProps {
  form: Pick<
    TaskBoardFormState['settings'],
    'reviewEnabled' | 'reviewMaxTurns' | 'reviewPromptTemplate'
  >;
  data: TaskBoardReviewSettingsData;
  disabled: boolean;
  actions: Pick<
    TaskBoardActions,
    | 'onSettingsReviewEnabledChange'
    | 'onSettingsReviewMaxTurnsInput'
    | 'onSettingsReviewPromptInput'
  >;
}

const REVIEW_ENABLED_ITEMS = [
  { value: '', label: 'Use team default' },
  { value: 'true', label: 'Enabled for this project' },
  { value: 'false', label: 'Disabled for this project' }
];

export const TaskBoardReviewSettings = ({
  form,
  data,
  disabled,
  actions
}: TaskBoardReviewSettingsProps): JSX.Element => (
  <TaskBoardSettingsSection
    title='Review automation'
    description='Implementation runs spawn PR review jobs from submitted pull requests. Override review enablement, follow-up passes, and review prompt.'
    hasOverrides={data.hasOverrides}
  >
    <div class='grid grid-cols-1 gap-4 md:grid-cols-2'>
      <div class='flex flex-col gap-2'>
        <Label for='board-review-enabled'>Review automation</Label>
        <Select
          id='board-review-enabled'
          value={form.reviewEnabled}
          onValueChange={actions.onSettingsReviewEnabledChange}
          disabled={disabled}
          items={REVIEW_ENABLED_ITEMS}
        />
      </div>
      <div class='flex flex-col gap-2'>
        <Label for='board-review-max-turns'>Review follow-up passes</Label>
        <Input
          id='board-review-max-turns'
          type='number'
          min={1}
          max={10}
          value={form.reviewMaxTurns}
          onInput={actions.onSettingsReviewMaxTurnsInput}
          disabled={disabled}
          placeholder='Use team default'
        />
      </div>
    </div>
    <div class='flex flex-col gap-2'>
      <Label for='board-review-prompt'>Review Prompt Template</Label>
      <TextArea
        id='board-review-prompt'
        value={form.reviewPromptTemplate}
        onInput={actions.onSettingsReviewPromptInput}
        rows={4}
        disabled={disabled}
        placeholder='Use team default review prompt'
      />
    </div>
  </TaskBoardSettingsSection>
);
