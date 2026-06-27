import type { JSX } from 'preact';
import { useMemo } from 'preact/hooks';
import type { SelectOption } from '../../../types/shared';
import { Input } from '../../shared/ui/Input.view';
import { Label } from '../../shared/ui/Label.view';
import { Select } from '../../shared/ui/Select.view';
import { TextArea } from '../../shared/ui/TextArea.view';
import type { TaskBoardActions, TaskBoardFormState } from '../types';
import { TaskBoardSettingsSection } from './TaskBoardSettingsSection.view';

interface TaskBoardReviewSettingsProps {
  form: Pick<
    TaskBoardFormState['settings'],
    'reviewEnabled' | 'reviewPickupColumn' | 'reviewMaxTurns' | 'reviewPromptTemplate'
  >;
  statusOptions: SelectOption[];
  disabled: boolean;
  actions: Pick<
    TaskBoardActions,
    | 'onSettingsReviewEnabledChange'
    | 'onSettingsReviewPickupColumnChange'
    | 'onSettingsReviewMaxTurnsInput'
    | 'onSettingsReviewPromptInput'
  >;
}

const REVIEW_ENABLED_ITEMS: SelectOption[] = [
  { value: '', label: 'Use team default' },
  { value: 'true', label: 'Enabled for this project' },
  { value: 'false', label: 'Disabled for this project' }
];

export const TaskBoardReviewSettings = ({
  form,
  statusOptions,
  disabled,
  actions
}: TaskBoardReviewSettingsProps): JSX.Element => {
  const reviewPickupColumnItems = useMemo(
    () => [{ value: '', label: 'Use column role or team default' }, ...statusOptions],
    [statusOptions]
  );

  return (
    <TaskBoardSettingsSection
      title='Review automation'
      description='Override review enablement, pickup column, follow-up passes, and review prompt.'
      hasOverrides={
        form.reviewEnabled !== '' ||
        form.reviewPickupColumn !== '' ||
        form.reviewMaxTurns.trim().length > 0 ||
        form.reviewPromptTemplate.trim().length > 0
      }
    >
      <div class='grid grid-cols-1 gap-4 md:grid-cols-2'>
        <div class='flex flex-col gap-2'>
          <Label for='board-settings-review-enabled'>Review automation</Label>
          <Select
            id='board-settings-review-enabled'
            value={form.reviewEnabled}
            onValueChange={actions.onSettingsReviewEnabledChange}
            disabled={disabled}
            items={REVIEW_ENABLED_ITEMS}
          />
        </div>
        <div class='flex flex-col gap-2'>
          <Label for='board-settings-review-column'>Review pickup column</Label>
          <Select
            id='board-settings-review-column'
            value={form.reviewPickupColumn}
            onValueChange={actions.onSettingsReviewPickupColumnChange}
            disabled={disabled}
            items={reviewPickupColumnItems}
          />
        </div>
        <div class='flex flex-col gap-2'>
          <Label for='board-settings-review-turns'>Review follow-up passes</Label>
          <Input
            id='board-settings-review-turns'
            type='number'
            min='1'
            placeholder='Use team default'
            value={form.reviewMaxTurns}
            disabled={disabled}
            onInput={actions.onSettingsReviewMaxTurnsInput}
          />
        </div>
      </div>
      <div class='flex flex-col gap-2'>
        <Label for='board-settings-review-prompt'>Review prompt template</Label>
        <TextArea
          id='board-settings-review-prompt'
          value={form.reviewPromptTemplate}
          rows={5}
          disabled={disabled}
          onInput={actions.onSettingsReviewPromptInput}
        />
      </div>
    </TaskBoardSettingsSection>
  );
};
