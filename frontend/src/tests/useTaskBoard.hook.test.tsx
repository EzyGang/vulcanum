import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { fireEvent, render, waitFor } from '@testing-library/preact';
import type { JSX } from 'preact';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import type { TaskBoardResponse } from '../types/task-board';

const serviceMocks = vi.hoisted(() => ({
  getTaskBoard: vi.fn(),
  listProjects: vi.fn(),
  listRepos: vi.fn()
}));

vi.mock('../services/task-board/task-board.service', () => ({
  getTaskBoard: serviceMocks.getTaskBoard
}));

vi.mock('../services/projects/projects.service', () => ({
  listProjects: serviceMocks.listProjects
}));

vi.mock('../services/github/github.service', () => ({
  listRepos: serviceMocks.listRepos
}));

vi.mock('../components/task-board/hooks/useTaskBoardCreate.hook', () => ({
  useTaskBoardCreate: () => ({
    dialogOpen: false,
    form: { title: '', body: '', status: '', createError: null },
    error: null,
    status: { creating: false },
    actions: {}
  })
}));

vi.mock('../components/task-board/hooks/useTaskBoardMovement.hook', () => ({
  useTaskBoardMovement: () => ({
    data: {
      selectedTask: null,
      visibleTaskCounts: {},
      actionMenuTaskId: null,
      actionMenuPosition: null,
      dropPreviewColumn: null
    },
    form: {},
    error: null,
    status: {
      movingTaskId: null,
      moving: false,
      updatingTask: false,
      updatingTaskLabel: false
    },
    actions: {}
  })
}));

vi.mock('../components/task-board/hooks/useTaskBoardSettings.hook', () => ({
  useTaskBoardSettings: () => ({
    data: {
      settingsDialogOpen: true,
      automationEnabled: false,
      dismissedHelpCards: []
    },
    form: {},
    error: null,
    status: {},
    actions: {}
  })
}));

vi.mock('../components/task-board/hooks/useTaskBoardViewModel.hook', () => ({
  useTaskBoardViewModel: () => ({
    data: {
      boardColumnCount: 1,
      columns: [],
      hiddenColumns: [],
      hasCustomColumnView: false
    },
    actions: {}
  })
}));

import { useTaskBoard } from '../components/task-board/hooks/useTaskBoard.hook';
import { selectedTaskProjectKey } from '../stores/task-board.store';

const makeBoardResponse = (name: string): TaskBoardResponse => ({
  providerId: 'provider-1',
  providerType: 'linear',
  board: {
    project: { id: 'project-1', name, slug: 'project-1' },
    columns: [],
    labels: []
  },
  projectUsage: {
    total: {
      tokensUsed: 0,
      inputTokens: 0,
      outputTokens: 0,
      cacheReadTokens: 0,
      cacheWriteTokens: 0,
      finishedRunsCount: 0,
      implementationRunsCount: 0,
      reviewRunsCount: 0,
      successfulRunsCount: 0,
      failedRunsCount: 0
    },
    thisWeek: {
      tokensUsed: 0,
      inputTokens: 0,
      outputTokens: 0,
      cacheReadTokens: 0,
      cacheWriteTokens: 0,
      finishedRunsCount: 0,
      implementationRunsCount: 0,
      reviewRunsCount: 0,
      successfulRunsCount: 0,
      failedRunsCount: 0
    }
  },
  taskAugmentations: []
});

const BoardHarness = (): JSX.Element => {
  const { data, status, actions } = useTaskBoard();

  return (
    <div>
      <span>{data.board?.project.name}</span>
      <span>{data.settingsDialogOpen ? 'Settings open' : 'Settings closed'}</span>
      <button
        type='button'
        aria-label='Refresh board'
        disabled={status.refreshing}
        onClick={actions.onRefreshBoard}
      />
    </div>
  );
};

beforeEach(() => {
  selectedTaskProjectKey.value = 'provider-1/project-1';
  serviceMocks.getTaskBoard.mockReset();
  serviceMocks.listProjects.mockReset().mockResolvedValue([]);
  serviceMocks.listRepos.mockReset().mockResolvedValue([]);
});

describe('useTaskBoard manual refresh', () => {
  it('refetches the selected board once and preserves local dialog state', async () => {
    let resolveRefresh: (response: TaskBoardResponse) => void = () => undefined;
    const refreshResponse = new Promise<TaskBoardResponse>((resolve) => {
      resolveRefresh = resolve;
    });
    serviceMocks.getTaskBoard
      .mockResolvedValueOnce(makeBoardResponse('Initial board'))
      .mockReturnValueOnce(refreshResponse);
    const queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false } }
    });
    const view = render(
      <QueryClientProvider client={queryClient}>
        <BoardHarness />
      </QueryClientProvider>
    );

    await waitFor(() => expect(view.getByText('Initial board')).toBeTruthy());

    const refreshButton = view.getByRole('button', { name: 'Refresh board' });
    fireEvent.click(refreshButton);

    await waitFor(() => expect(refreshButton).toHaveProperty('disabled', true));
    fireEvent.click(refreshButton);
    expect(serviceMocks.getTaskBoard).toHaveBeenCalledTimes(2);
    expect(serviceMocks.getTaskBoard).toHaveBeenLastCalledWith('provider-1', 'project-1');

    resolveRefresh(makeBoardResponse('Refreshed board'));

    await waitFor(() => expect(view.getByText('Refreshed board')).toBeTruthy());
    expect(refreshButton).toHaveProperty('disabled', false);
    expect(view.getByText('Settings open')).toBeTruthy();
  });
});
