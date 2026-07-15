import type { JSX } from 'preact';
import { Button } from '../../shared/ui/Button.view';
import { Input } from '../../shared/ui/Input.view';
import { Label } from '../../shared/ui/Label.view';
import { TextArea } from '../../shared/ui/TextArea.view';
import type { TaskBoardActions, TaskBoardFormState, TaskBoardProjectSettingsData } from '../types';
import { TaskBoardSettingsSection } from './TaskBoardSettingsSection.view';

interface TaskBoardProjectSettingsProps {
  form: Pick<TaskBoardFormState['settings'], 'promptTemplate' | 'agentsMd' | 'maxInProgressTasks'>;
  data: TaskBoardProjectSettingsData;
  disabled: boolean;
  actions: Pick<
    TaskBoardActions,
    | 'onSettingsPromptInput'
    | 'onResetSettingsPrompt'
    | 'onSettingsAgentsInput'
    | 'onSettingsMaxInProgressInput'
  >;
}

export const TaskBoardProjectSettings = ({
  form,
  data,
  disabled,
  actions
}: TaskBoardProjectSettingsProps): JSX.Element => (
  <TaskBoardSettingsSection
    title='Project overrides'
    description='Leave fields empty to inherit the team defaults.'
    hasOverrides={data.hasOverrides}
  >
    <div class='flex flex-col gap-2'>
      <div class='flex items-center justify-between gap-3'>
        <Label for='board-settings-prompt'>Prompt template</Label>
        <Button
          type='button'
          variant='ghost'
          disabled={disabled || !form.promptTemplate.trim()}
          onClick={actions.onResetSettingsPrompt}
        >
          Reset to team default
        </Button>
      </div>
      <TextArea
        id='board-settings-prompt'
        value={form.promptTemplate}
        rows={5}
        disabled={disabled}
        onInput={actions.onSettingsPromptInput}
      />
    </div>
    <div class='flex flex-col gap-2'>
      <Label for='board-settings-agents'>Agents.md</Label>
      <TextArea
        id='board-settings-agents'
        value={form.agentsMd}
        rows={5}
        disabled={disabled}
        onInput={actions.onSettingsAgentsInput}
      />
    </div>
    <div class='flex flex-col gap-2 md:max-w-xs'>
      <Label for='board-settings-max-in-progress'>Max in-progress tasks</Label>
      <Input
        id='board-settings-max-in-progress'
        type='number'
        min='1'
        placeholder='Use team default'
        value={form.maxInProgressTasks}
        disabled={disabled}
        onInput={actions.onSettingsMaxInProgressInput}
      />
    </div>
  </TaskBoardSettingsSection>
);
