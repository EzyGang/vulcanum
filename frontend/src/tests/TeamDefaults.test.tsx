import { signal } from '@preact/signals';
import { fireEvent, render } from '@testing-library/preact';
import type { ComponentProps } from 'preact';
import { describe, expect, it, vi } from 'vitest';
import { TeamDefaultsView } from '../components/team-defaults/ui/TeamDefaults.view';

type TeamDefaultsProps = ComponentProps<typeof TeamDefaultsView>;

const makeProps = (): TeamDefaultsProps => ({
  section: 'defaults',
  data: {
    promptTemplate: signal('Custom implementation prompt'),
    promptTemplateInherited: signal(false),
    agentsMd: signal(''),
    primaryModelProviderKey: signal(''),
    primaryModelId: signal(''),
    smallModelProviderKey: signal(''),
    smallModelId: signal(''),
    reviewEnabled: signal(false),
    reviewMaxTurns: signal(1),
    reviewPromptTemplate: signal('Custom review prompt'),
    reviewPromptTemplateInherited: signal(false),
    maxInProgressTasks: signal(1),
    agentBackend: signal('opencode'),
    agentBackendItems: [],
    connectedProviderItems: [],
    primaryModelItems: [],
    smallModelItems: []
  },
  status: {
    loading: false,
    saving: false,
    error: signal(null)
  },
  actions: {
    onPromptTemplateInput: vi.fn(),
    onResetPromptTemplate: vi.fn(),
    onAgentsMdInput: vi.fn(),
    onPrimaryProviderChange: vi.fn(),
    onPrimaryModelChange: vi.fn(),
    onSmallProviderChange: vi.fn(),
    onSmallModelChange: vi.fn(),
    onAgentBackendChange: vi.fn(),
    onReviewEnabledChange: vi.fn(),
    onReviewMaxTurnsInput: vi.fn(),
    onReviewPromptTemplateInput: vi.fn(),
    onResetReviewPromptTemplate: vi.fn(),
    onMaxInProgressTasksInput: vi.fn(),
    onSubmit: vi.fn()
  }
});

describe('TeamDefaultsView', () => {
  it('resets both team prompt overrides', () => {
    const props = makeProps();
    const { getAllByText } = render(<TeamDefaultsView {...props} />);
    const resetButtons = getAllByText('Reset to system default');

    fireEvent.click(resetButtons[0]);
    fireEvent.click(resetButtons[1]);

    expect(props.actions.onResetPromptTemplate).toHaveBeenCalledOnce();
    expect(props.actions.onResetReviewPromptTemplate).toHaveBeenCalledOnce();
  });

  it('disables reset controls for inherited prompts', () => {
    const props = makeProps();
    props.data.promptTemplateInherited.value = true;
    props.data.reviewPromptTemplateInherited.value = true;
    const { getAllByText } = render(<TeamDefaultsView {...props} />);

    for (const button of getAllByText('Reset to system default')) {
      expect((button as HTMLButtonElement).disabled).toBe(true);
    }
  });
});
