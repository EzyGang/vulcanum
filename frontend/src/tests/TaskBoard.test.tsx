import { fireEvent, render } from '@testing-library/preact';
import type { ComponentChildren } from 'preact';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import {
  normalizeTaskBoardColumnPreferences,
  readDismissedHelpCards,
  readTaskBoardColumnPreferences,
  writeDismissedHelpCards,
  writeTaskBoardColumnPreferences
} from '../components/task-board/hooks/taskBoard.helpers';

vi.mock('../components/shared/ui/Select.view', () => ({
  Select: ({
    items,
    value,
    onValueChange,
    placeholder,
    disabled,
    id
  }: {
    items: { value: string; label: string }[];
    value: string;
    onValueChange: (value: string) => void;
    placeholder?: string;
    disabled?: boolean;
    id?: string;
  }) => (
    <select
      id={id}
      value={value}
      disabled={disabled}
      aria-label={placeholder}
      onInput={(event) => onValueChange((event.target as HTMLSelectElement).value)}
    >
      {items.map((item) => (
        <option key={item.value} value={item.value}>
          {item.label}
        </option>
      ))}
    </select>
  )
}));

vi.mock('../components/shared/ui/Checkbox.view', () => ({
  Checkbox: ({
    id,
    checked,
    disabled,
    onCheckedChange
  }: {
    id?: string;
    checked?: boolean;
    disabled?: boolean;
    onCheckedChange?: (checked: boolean) => void;
  }) => (
    <input
      id={id}
      type='checkbox'
      checked={checked}
      disabled={disabled}
      onChange={(event) => onCheckedChange?.((event.target as HTMLInputElement).checked)}
    />
  )
}));

vi.mock('../components/shared/ui/Dialog.view', () => {
  const Dialog = ({
    children,
    open
  }: {
    children: ComponentChildren;
    open?: boolean;
    onOpenChange?: (open: boolean) => void;
  }) => (open ? <div>{children}</div> : null);
  Dialog.Portal = ({ children }: { children: ComponentChildren }) => <div>{children}</div>;
  Dialog.Backdrop = () => <div />;
  Dialog.Popup = ({ children }: { children: ComponentChildren }) => <div>{children}</div>;
  Dialog.Title = ({ children }: { children: ComponentChildren }) => <h2>{children}</h2>;
  Dialog.Description = ({ children }: { children: ComponentChildren }) => <p>{children}</p>;
  Dialog.Close = ({ children }: { children: ComponentChildren }) => <div>{children}</div>;
  return { Dialog };
});

vi.mock('../components/shared/ui/Tooltip.view', () => {
  const Tooltip = ({ children }: { children: ComponentChildren }) => <div>{children}</div>;
  Tooltip.Trigger = ({
    children,
    class: className
  }: {
    children: ComponentChildren;
    class?: string;
  }) => <div class={className}>{children}</div>;
  Tooltip.Popup = ({ children }: { children: ComponentChildren }) => <div>{children}</div>;
  return { Tooltip };
});

import { formatTaskDisplayId } from '../components/task-board/hooks/taskBoardViewModel.support';
import type { TaskBoardViewProps } from '../components/task-board/types';
import { TaskBoardView } from '../components/task-board/ui/TaskBoard.view';
import type { TaskBoardTaskAugmentation } from '../types/task-board';

const makeTask = (id: string, title = `Task ${id}`) => ({
  id,
  title,
  projectId: 'project-1',
  description: 'Hidden task body',
  status: 'to-do',
  priority: 'low',
  number: 10,
  projectSlug: 'act',
  assigneeName: null,
  createdAt: '2026-01-01T00:00:00Z',
  updatedAt: null,
  labels: [{ id: 'label-1', name: 'Bug', color: '#ef4444' }]
});

const makeAugmentation = (
  overrides: Partial<TaskBoardTaskAugmentation> = {}
): TaskBoardTaskAugmentation => ({
  externalTaskRef: 'task-1',
  tokensUsed: 1545,
  inputTokens: 1200,
  outputTokens: 345,
  cacheReadTokens: 80,
  cacheWriteTokens: 12,
  finishedRunsCount: 2,
  updatedAt: '2026-01-02T00:00:00Z',
  ...overrides
});

const makeProps = (
  augmentationsByTaskRef: Readonly<Record<string, TaskBoardTaskAugmentation>> = {}
): TaskBoardViewProps => {
  const statusOptions = [
    { value: 'to-do', label: 'To Do' },
    { value: 'in-progress', label: 'In Progress' },
    { value: 'done', label: 'Done' }
  ];
  const board = {
    project: { id: 'project-1', name: 'Proxy Board', slug: 'proxy-board' },
    columns: [
      {
        id: 'column-1',
        name: 'To Do',
        slug: 'to-do',
        isFinal: false,
        tasks: [makeTask('task-1', 'Create proxy API')]
      },
      { id: 'column-2', name: 'Done', slug: 'done', isFinal: true, tasks: [] }
    ],
    labels: [{ id: 'label-1', name: 'Bug', color: '#ef4444' }]
  };
  const actions: TaskBoardViewProps['actions'] = {
    onTitleInput: vi.fn(),
    onBodyInput: vi.fn(),
    onStatusChange: vi.fn(),
    onSubmitTask: vi.fn((event: Event) => event.preventDefault()),
    onMoveTask: vi.fn(),
    onEditTaskTitleInput: vi.fn(),
    onEditTaskBodyInput: vi.fn(),
    onSubmitTaskEdit: vi.fn((event: Event) => event.preventDefault()),
    onToggleTaskLabel: vi.fn(),
    onDeleteLabel: vi.fn(),
    onToggleRepo: vi.fn(),
    onFilterRepos: vi.fn(),
    onSettingsPromptInput: vi.fn(),
    onSettingsAgentsInput: vi.fn(),
    onSettingsReviewEnabledChange: vi.fn(),
    onSettingsReviewMaxTurnsInput: vi.fn(),
    onSettingsReviewPromptInput: vi.fn(),
    onSettingsMaxInProgressInput: vi.fn(),
    onSubmitSettings: vi.fn((event: Event) => event.preventDefault()),
    onSetColumnRole: vi.fn(),
    onToggleAutomation: vi.fn(),
    onDismissHelpCard: vi.fn(),
    onOpenTask: vi.fn(),
    onCloseTask: vi.fn(),
    onTaskDetailsOpenChange: vi.fn(),
    onDragStart: vi.fn(),
    onDragOverStatus: vi.fn((event: DragEvent) => event.preventDefault()),
    onDragEnd: vi.fn(),
    onDropOnStatus: vi.fn(),
    onOpenCreateTask: vi.fn(),
    onCloseCreateTask: vi.fn(),
    onCreateDialogOpenChange: vi.fn(),
    onOpenSettings: vi.fn(),
    onCloseSettings: vi.fn(),
    onSettingsDialogOpenChange: vi.fn(),
    onOpenTaskMenu: vi.fn((event: MouseEvent) => event.preventDefault()),
    onCloseTaskMenu: vi.fn(),
    onLoadMoreColumn: vi.fn(),
    onColumnScroll: vi.fn(),
    onPickupColumnChange: vi.fn(),
    onProgressColumnChange: vi.fn(),
    onReviewColumnChange: vi.fn(),
    onDoneColumnChange: vi.fn(),
    onShowColumn: vi.fn(),
    onHideColumn: vi.fn(),
    onMoveColumnLeft: vi.fn(),
    onMoveColumnRight: vi.fn(),
    onResetColumnView: vi.fn()
  };
  const data: TaskBoardViewProps['data'] = {
    selectedProjectKey: 'provider-1/project-1',
    board,
    get boardColumnCount() {
      return Math.max(board.columns.length, 1);
    },
    get columns() {
      return board.columns.map((column) => {
        const visibleCount = data.visibleTaskCounts[column.slug] ?? 20;
        const visibleTasks = column.tasks.slice(0, visibleCount);
        const activeRoles = ['pickup', 'progress', 'review', 'done'] as const;
        const columnRoles = data.columnRoles;
        const activeColumnRoles = activeRoles
          .filter(
            (role) =>
              (role === 'pickup' && columnRoles.pickupColumn === column.slug) ||
              (role === 'progress' && columnRoles.progressColumn === column.slug) ||
              (role === 'review' && columnRoles.reviewColumn === column.slug) ||
              (role === 'done' && columnRoles.doneColumn === column.slug)
          )
          .map((role) => ({ role }));

        return {
          column,
          visibleTasks: visibleTasks.map((task) => ({
            augmentation: augmentationsByTaskRef[task.id] ?? null,
            task,
            displayId: formatTaskDisplayId(task),
            createdAtLabel: new Date(task.createdAt).toLocaleDateString(),
            moving: false,
            menuOpen: data.actionMenuTaskId === task.id,
            menuStyle:
              data.actionMenuTaskId === task.id && data.actionMenuPosition
                ? { left: `${data.actionMenuPosition.x}px`, top: `${data.actionMenuPosition.y}px` }
                : undefined,
            moveActions: statusOptions
              .filter((option) => option.value !== task.status)
              .map((option) => ({
                value: option.value,
                label: option.label,
                onClick: () => actions.onMoveTask(task.id, option.value)
              })),
            onClick: () => actions.onOpenTask(task),
            onOpenMenu: (event: MouseEvent) => actions.onOpenTaskMenu(event, task.id),
            onDragStart: () => actions.onDragStart(task.id, task.status),
            onDragEnd: actions.onDragEnd,
            onKeyDown: () => actions.onOpenTask(task),
            onStopMenuClick: vi.fn()
          })),
          taskCount: column.tasks.length,
          activeRoles: activeColumnRoles,
          hasMoreTasks: visibleTasks.length < column.tasks.length,
          dropPreviewActive: data.dropPreviewColumn === column.slug,
          roleMenu: {
            buttonLabel: `Column role settings for ${column.name}`,
            menuLabel: `Column roles for ${column.name}`,
            open: column.slug === 'done',
            disabled: false,
            onToggle: vi.fn(),
            onStopClick: vi.fn(),
            items: activeRoles.map((role) => ({
              role,
              label: `Set ${role === 'progress' ? 'In progress' : role[0].toUpperCase()}${role === 'progress' ? '' : role.slice(1)}`,
              help: 'Role help',
              active: false,
              disabled: false,
              onClick: () => actions.onSetColumnRole(column.slug, role)
            }))
          },
          viewControls: {
            canMoveLeft: column.slug !== board.columns[0]?.slug,
            canMoveRight: column.slug !== board.columns[board.columns.length - 1]?.slug,
            onHide: () => actions.onHideColumn(column.slug),
            onMoveLeft: () => actions.onMoveColumnLeft(column.slug),
            onMoveRight: () => actions.onMoveColumnRight(column.slug)
          },
          onDragOver: (event: DragEvent) => actions.onDragOverStatus(event, column.slug),
          onDrop: (event: DragEvent) => actions.onDropOnStatus(event, column.slug),
          onScroll: (event: Event) => actions.onColumnScroll(event, column.slug),
          onLoadMore: () => actions.onLoadMoreColumn(column.slug)
        };
      });
    },
    hiddenColumns: [],
    hasCustomColumnView: false,
    helpCards: [],
    automationLabel: 'Automation off',
    statusOptions,
    repoItems: [{ value: 'owner/repo', label: 'owner/repo' }],
    selectedRepoNames: [],
    selectedTask: null,
    get selectedTaskAugmentation() {
      return data.selectedTask ? (augmentationsByTaskRef[data.selectedTask.id] ?? null) : null;
    },
    availableLabels: board.labels,
    createDialogOpen: false,
    settingsDialogOpen: false,
    actionMenuTaskId: null,
    actionMenuPosition: null,
    visibleTaskCounts: { 'to-do': 20, done: 20 },
    columnRoles: {
      pickupColumn: 'to-do',
      progressColumn: 'to-do',
      reviewColumn: 'done',
      doneColumn: 'done'
    },
    dropPreviewColumn: null,
    automationEnabled: false,
    dismissedHelpCards: [],
    get repositorySettings() {
      const selectedRepoNames = data.selectedRepoNames;
      const selectedRepos = selectedRepoNames.map((repoFullName) => ({
        value: repoFullName,
        label: repoFullName,
        checked: true,
        onToggle: () => actions.onToggleRepo(repoFullName)
      }));
      const filteredRepos = data.repoItems
        .filter((repo) => !selectedRepoNames.includes(repo.value))
        .map((repo) => ({
          ...repo,
          checked: false,
          onToggle: () => actions.onToggleRepo(repo.value)
        }));

      return {
        filter: '',
        selectedRepos,
        filteredRepos,
        hasRepos: data.repoItems.length > 0,
        hasSelectedRepos: selectedRepos.length > 0,
        hasFilteredRepos: filteredRepos.length > 0,
        hasOverrides: selectedRepoNames.length > 0
      };
    },
    columnSettings: { hasOptions: true, roleSelects: [] },
    projectSettings: { hasOverrides: false },
    reviewSettings: {
      hasOverrides: false
    },
    get selectedTaskCreatedAtLabel() {
      return data.selectedTask ? new Date(data.selectedTask.createdAt).toLocaleString() : null;
    },
    get selectedTaskMoveActions() {
      return data.selectedTask
        ? statusOptions
            .filter((option) => option.value !== data.selectedTask?.status)
            .map((option) => ({
              value: option.value,
              label: option.label,
              onClick: () =>
                data.selectedTask && actions.onMoveTask(data.selectedTask.id, option.value)
            }))
        : [];
    }
  };

  return {
    data,
    form: {
      title: '',
      body: '',
      status: 'to-do',
      createError: null,
      serverError: null,
      editTitle: '',
      editBody: '',
      editLabelIds: [],
      editError: null,
      settings: {
        promptTemplate: '',
        agentsMd: '',
        reviewEnabled: '',
        reviewMaxTurns: '',
        reviewPromptTemplate: '',
        maxInProgressTasks: ''
      }
    },
    status: {
      loading: false,
      error: null,
      creating: false,
      movingTaskId: null,
      moving: false,
      updatingTask: false,
      updatingTaskLabel: false,
      reposLoading: false,
      connectingRepos: false,
      connected: true,
      savingSettings: false,
      configuringColumns: false,
      savingAutomation: false,
      settingsDisabled: false,
      repoControlsDisabled: false
    },
    actions
  };
};

const requireBoard = (props: TaskBoardViewProps) => {
  if (!props.data.board) {
    throw new Error('Expected board fixture');
  }

  return props.data.board;
};

beforeEach(() => {
  localStorage.clear();
});

describe('taskBoard.helpers', () => {
  it('persists dismissed lifecycle labels with other help cards', () => {
    writeDismissedHelpCards(['proxy', 'lifecycle-labels']);

    expect(readDismissedHelpCards()).toEqual(['proxy', 'lifecycle-labels']);
  });

  it('persists column view preferences per board in local storage', () => {
    writeTaskBoardColumnPreferences('provider-1/project-1', {
      hiddenColumnSlugs: ['done'],
      columnOrder: ['done', 'to-do']
    });

    expect(readTaskBoardColumnPreferences('provider-1/project-1')).toEqual({
      hiddenColumnSlugs: ['done'],
      columnOrder: ['done', 'to-do']
    });
    expect(readTaskBoardColumnPreferences('provider-1/project-2')).toEqual({
      hiddenColumnSlugs: [],
      columnOrder: []
    });
  });

  it('normalizes column preferences against provider columns', () => {
    const board = requireBoard(makeProps());

    expect(
      normalizeTaskBoardColumnPreferences(board.columns, {
        hiddenColumnSlugs: ['missing', 'done', 'done'],
        columnOrder: ['missing', 'done', 'done']
      })
    ).toEqual({
      hiddenColumnSlugs: ['done'],
      columnOrder: ['done', 'to-do']
    });
  });
});

describe('TaskBoard.view', () => {
  it('renders provider columns and tasks without task bodies', () => {
    const props = makeProps();
    const { queryByText, getAllByText, getByText } = render(<TaskBoardView {...props} />);

    expect(getByText('Proxy Board')).toBeTruthy();
    expect(getByText('Create proxy API')).toBeTruthy();
    expect(getByText('ACT-10')).toBeTruthy();
    expect(queryByText('Hidden task body')).toBeNull();
    expect(getAllByText('To Do').length).toBeGreaterThan(0);
    expect(getAllByText('Done').length).toBeGreaterThan(0);
    expect(getAllByText('Review').length).toBeGreaterThan(0);
  });

  it('sends column hide and reorder actions from column headers', () => {
    const props = makeProps();
    const { getByLabelText } = render(<TaskBoardView {...props} />);

    fireEvent.click(getByLabelText('Hide To Do column'));
    fireEvent.click(getByLabelText('Move Done column left'));

    expect(props.actions.onHideColumn).toHaveBeenCalledWith('to-do');
    expect(props.actions.onMoveColumnLeft).toHaveBeenCalledWith('done');
  });

  it('restores hidden columns and can reset the local board view', () => {
    const props = makeProps();
    const board = requireBoard(props);
    const doneColumn = board.columns[1];
    props.data.hasCustomColumnView = true;
    props.data.hiddenColumns = doneColumn
      ? [
          {
            column: doneColumn,
            onShow: () => props.actions.onShowColumn(doneColumn.slug)
          }
        ]
      : [];

    const { getByText, queryByText } = render(<TaskBoardView {...props} />);

    expect(queryByText(/saved in this browser only/i)).toBeNull();
    fireEvent.click(getByText('Show Done'));
    fireEvent.click(getByText('Reset view'));

    expect(props.actions.onShowColumn).toHaveBeenCalledWith('done');
    expect(props.actions.onResetColumnView).toHaveBeenCalledOnce();
  });

  it('opens task creation from the board button', () => {
    const props = makeProps();
    const { getByText } = render(<TaskBoardView {...props} />);

    fireEvent.click(getByText('Create task'));

    expect(props.actions.onOpenCreateTask).toHaveBeenCalledOnce();
  });

  it('submits task creation from the creation modal', () => {
    const props = makeProps();
    props.data.createDialogOpen = true;
    const { getAllByText } = render(<TaskBoardView {...props} />);
    const createForm = getAllByText('Create task')
      .map((element) => element.closest('form'))
      .find(Boolean);

    fireEvent.submit(createForm as Element);

    expect(props.actions.onSubmitTask).toHaveBeenCalledOnce();
  });

  it('opens a task action menu from the visible action button', () => {
    const props = makeProps();
    const { queryByText, getByLabelText } = render(<TaskBoardView {...props} />);

    expect(queryByText('Mark Done')).toBeNull();
    fireEvent.click(getByLabelText('Actions for Create proxy API'));

    expect(props.actions.onOpenTaskMenu).toHaveBeenCalledWith(expect.any(Object), 'task-1');
  });

  it('moves a task from the action menu', () => {
    const props = makeProps();
    props.data.actionMenuTaskId = 'task-1';
    const { getByText } = render(<TaskBoardView {...props} />);

    fireEvent.click(getByText('Mark Done'));

    expect(props.actions.onMoveTask).toHaveBeenCalledWith('task-1', 'done');
  });

  it('does not open the action menu from card right-click', () => {
    const props = makeProps();
    const { getByText } = render(<TaskBoardView {...props} />);
    const card = getByText('Create proxy API').closest('article');

    fireEvent.contextMenu(card as Element);

    expect(props.actions.onOpenTaskMenu).not.toHaveBeenCalled();
  });

  it('opens task details through the provided action', () => {
    const props = makeProps();
    const board = requireBoard(props);
    const { getByText } = render(<TaskBoardView {...props} />);

    fireEvent.click(getByText('Create proxy API'));

    expect(props.actions.onOpenTask).toHaveBeenCalledWith(board.columns[0].tasks[0]);
  });

  it('deletes a provider label from the task details dialog', () => {
    const props = makeProps();
    const board = requireBoard(props);
    props.data.selectedTask = board.columns[0].tasks[0] ?? null;

    const { getByLabelText } = render(<TaskBoardView {...props} />);
    fireEvent.click(getByLabelText('Delete label Bug'));

    expect(props.actions.onDeleteLabel).toHaveBeenCalledWith('label-1');
  });

  it('shows editable task body in the details modal', () => {
    const props = makeProps();
    const board = requireBoard(props);
    props.data.selectedTask = board.columns[0].tasks[0];
    props.form.editTitle = board.columns[0].tasks[0].title;
    props.form.editBody = board.columns[0].tasks[0].description ?? '';
    const { getByDisplayValue } = render(<TaskBoardView {...props} />);

    expect(getByDisplayValue('Hidden task body')).toBeTruthy();
  });

  it('renders cumulative usage on task cards', () => {
    const props = makeProps({
      'task-1': makeAugmentation()
    });
    const { getByLabelText } = render(<TaskBoardView {...props} />);

    const usage = getByLabelText('Cumulative usage');

    expect(usage.textContent).toContain('Usage');
    expect(usage.textContent).toContain('2 finished runs');
    expect(usage.textContent).toContain('1.2K');
    expect(usage.textContent).toContain('345');
  });

  it('keeps task cards without cumulative usage quiet', () => {
    const props = makeProps();
    const { queryByLabelText, queryByText } = render(<TaskBoardView {...props} />);

    expect(queryByLabelText('Cumulative usage')).toBeNull();
    expect(queryByText('No usage recorded yet.')).toBeNull();
  });

  it('renders cumulative usage in the task details modal', () => {
    const props = makeProps({
      'task-1': makeAugmentation({
        tokensUsed: 1025,
        inputTokens: 900,
        outputTokens: 125,
        cacheReadTokens: 0,
        cacheWriteTokens: 0,
        finishedRunsCount: 1
      })
    });
    const board = requireBoard(props);
    props.data.selectedTask = board.columns[0].tasks[0];

    const { getByText } = render(<TaskBoardView {...props} />);

    const usageSection = getByText('Cumulative usage').closest('section');

    expect(usageSection).toBeTruthy();
    expect(usageSection?.textContent).toContain('1 finished run');
    expect(usageSection?.textContent).toContain('900');
    expect(usageSection?.textContent).toContain('125');
  });

  it('shows the cumulative usage empty state in the task details modal', () => {
    const props = makeProps();
    const board = requireBoard(props);
    props.data.selectedTask = board.columns[0].tasks[0];

    const { getByText } = render(<TaskBoardView {...props} />);

    expect(getByText('Cumulative usage')).toBeTruthy();
    expect(getByText('No usage recorded yet.')).toBeTruthy();
  });

  it('opens board settings from the icon button', () => {
    const props = makeProps();
    const { getByLabelText } = render(<TaskBoardView {...props} />);

    fireEvent.click(getByLabelText('Board settings'));

    expect(props.actions.onOpenSettings).toHaveBeenCalledOnce();
  });

  it('connects a repository through the settings modal', () => {
    const props = makeProps();
    props.data.settingsDialogOpen = true;
    const { getByLabelText } = render(<TaskBoardView {...props} />);

    fireEvent.click(getByLabelText('owner/repo'));

    expect(props.actions.onToggleRepo).toHaveBeenCalledWith('owner/repo');
  });

  it('submits project overrides from the settings modal', () => {
    const props = makeProps();
    props.data.settingsDialogOpen = true;
    props.form.settings.promptTemplate = 'Use the pinned repositories.';
    const { getByText } = render(<TaskBoardView {...props} />);
    const settingsForm = getByText('Save settings').closest('form');

    fireEvent.submit(settingsForm as Element);

    expect(props.actions.onSubmitSettings).toHaveBeenCalledOnce();
  });

  it('keeps board column roles out of the settings modal', () => {
    const props = makeProps();
    props.data.settingsDialogOpen = true;
    const { queryByText } = render(<TaskBoardView {...props} />);

    expect(queryByText('Board columns')).toBeNull();
  });

  it('pins selected repositories and filters available repositories', () => {
    const props = makeProps();
    props.data.settingsDialogOpen = true;
    props.data.repoItems = [
      { value: 'owner/selected', label: 'owner/selected' },
      { value: 'team/api', label: 'team/api' }
    ];
    props.data.selectedRepoNames = ['owner/selected'];
    const { getByLabelText } = render(<TaskBoardView {...props} />);

    expect((getByLabelText('owner/selected') as HTMLInputElement).checked).toBe(true);

    const repoFilterInput = getByLabelText('Filter repositories') as HTMLInputElement;
    repoFilterInput.value = 'team';
    fireEvent.input(repoFilterInput);

    expect(getByLabelText('owner/selected')).toBeTruthy();
    expect(getByLabelText('team/api')).toBeTruthy();
    expect(props.actions.onFilterRepos).toHaveBeenCalledWith(expect.any(Object));
  });

  it('loads more tasks from a paginated column', () => {
    const props = makeProps();
    const board = requireBoard(props);
    board.columns[0].tasks = Array.from({ length: 21 }, (_, index) =>
      makeTask(`task-${index}`, `Task ${index}`)
    );
    props.data.visibleTaskCounts = { 'to-do': 20, done: 20 };
    const { getByText, queryByText } = render(<TaskBoardView {...props} />);

    expect(getByText('Task 19')).toBeTruthy();
    expect(queryByText('Task 20')).toBeNull();
    fireEvent.click(getByText('Load more'));

    expect(props.actions.onLoadMoreColumn).toHaveBeenCalledWith('to-do');
  });

  it('sets Review and Done roles from a column header menu', () => {
    const props = makeProps();
    const { getByLabelText, getByText } = render(<TaskBoardView {...props} />);

    fireEvent.click(getByLabelText('Column role settings for Done'));
    fireEvent.click(getByText('Set Review'));
    fireEvent.click(getByText('Set Done'));

    expect(props.actions.onSetColumnRole).toHaveBeenCalledWith('done', 'review');
    expect(props.actions.onSetColumnRole).toHaveBeenCalledWith('done', 'done');
  });

  it('shows a drop preview when hovering another column', () => {
    const props = makeProps();
    props.data.dropPreviewColumn = 'done';
    const { getByText } = render(<TaskBoardView {...props} />);

    expect(getByText('Drop to move into Done.')).toBeTruthy();
  });

  it('toggles project automation from the board header', () => {
    const props = makeProps();
    const { getByText } = render(<TaskBoardView {...props} />);

    fireEvent.click(getByText('Automation off'));

    expect(props.actions.onToggleAutomation).toHaveBeenCalledOnce();
  });

  it('warns when automation is on without pinned repositories', () => {
    const props = makeProps();
    props.data.automationEnabled = true;
    props.data.automationLabel = 'Automation on';
    const { getByText } = render(<TaskBoardView {...props} />);

    expect(getByText('Automation has no repositories set')).toBeTruthy();
    fireEvent.click(getByText('Set repositories'));

    expect(props.actions.onOpenSettings).toHaveBeenCalledOnce();
  });

  it('hides the repository warning after a repository is pinned', () => {
    const props = makeProps();
    props.data.automationEnabled = true;
    props.data.selectedRepoNames = ['owner/repo'];
    const { queryByText } = render(<TaskBoardView {...props} />);

    expect(queryByText('Automation has no repositories set')).toBeNull();
  });

  it('dismisses lifecycle labels through the help card action', () => {
    const props = makeProps();
    const { getByLabelText } = render(<TaskBoardView {...props} />);

    fireEvent.click(getByLabelText('Dismiss Lifecycle labels help'));

    expect(props.actions.onDismissHelpCard).toHaveBeenCalledWith('lifecycle-labels');
  });

  it('hides lifecycle labels after dismissal', () => {
    const props = makeProps();
    props.data.dismissedHelpCards = ['lifecycle-labels'];
    const { queryByText } = render(<TaskBoardView {...props} />);

    expect(queryByText('Lifecycle labels')).toBeNull();
  });

  it('drops a task onto a column through the provided action', () => {
    const props = makeProps();
    const { getByText } = render(<TaskBoardView {...props} />);
    const doneColumn = getByText('Drop tasks here or create a new one for this column.').closest(
      'section'
    );

    fireEvent.drop(doneColumn as Element);

    expect(props.actions.onDropOnStatus).toHaveBeenCalledWith(expect.any(Object), 'done');
  });
});
