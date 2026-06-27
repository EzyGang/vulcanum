import { fireEvent, render } from '@testing-library/preact';
import type { ComponentChildren } from 'preact';
import { describe, expect, it, vi } from 'vitest';

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

import { TaskBoardView } from '../components/task-board/ui/TaskBoard.view';
import type { TaskBoardTask } from '../types/task-board';

const makeTask = (id: string, title = `Task ${id}`) => ({
  id,
  title,
  projectId: 'project-1',
  description: 'Hidden task body',
  status: 'to-do',
  priority: 'low',
  number: 12,
  projectSlug: 'proxy-board',
  assigneeName: null,
  createdAt: '2026-01-01T00:00:00Z',
  updatedAt: null
});

const makeProps = () => ({
  data: {
    selectedProjectKey: 'provider-1/project-1',
    statusOptions: [
      { value: 'to-do', label: 'To Do' },
      { value: 'in-progress', label: 'In Progress' },
      { value: 'done', label: 'Done' }
    ],
    board: {
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
      ]
    },
    repoItems: [{ value: 'owner/repo', label: 'owner/repo' }],
    selectedRepoNames: [] as string[],
    selectedTask: null as TaskBoardTask | null,
    createDialogOpen: false,
    settingsDialogOpen: false,
    actionMenuTaskId: null as string | null,
    visibleTaskCounts: { 'to-do': 20, done: 20 },
    columnRoles: {
      pickupColumn: 'to-do',
      progressColumn: 'to-do',
      targetColumn: 'done',
      reviewPickupColumn: null
    }
  },
  form: {
    title: '',
    body: '',
    status: 'to-do',
    createError: null,
    serverError: null,
    settings: {
      promptTemplate: '',
      agentsMd: '',
      reviewEnabled: '',
      reviewPickupColumn: '',
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
    reposLoading: false,
    connectingRepos: false,
    connected: true,
    savingSettings: false,
    configuringColumns: false
  },
  actions: {
    onTitleInput: vi.fn(),
    onBodyInput: vi.fn(),
    onStatusChange: vi.fn(),
    onSubmitTask: vi.fn((event: Event) => event.preventDefault()),
    onMoveTask: vi.fn(),
    onToggleRepo: vi.fn(),
    onSettingsPromptInput: vi.fn(),
    onSettingsAgentsInput: vi.fn(),
    onSettingsReviewEnabledChange: vi.fn(),
    onSettingsReviewPickupColumnChange: vi.fn(),
    onSettingsReviewMaxTurnsInput: vi.fn(),
    onSettingsReviewPromptInput: vi.fn(),
    onSettingsMaxInProgressInput: vi.fn(),
    onSubmitSettings: vi.fn((event: Event) => event.preventDefault()),
    onSetColumnRole: vi.fn(),
    onOpenTask: vi.fn(),
    onCloseTask: vi.fn(),
    onDragStart: vi.fn(),
    onDragOver: vi.fn((event: DragEvent) => event.preventDefault()),
    onDropOnStatus: vi.fn(),
    onOpenCreateTask: vi.fn(),
    onCloseCreateTask: vi.fn(),
    onOpenSettings: vi.fn(),
    onCloseSettings: vi.fn(),
    onOpenTaskMenu: vi.fn((event: MouseEvent) => event.preventDefault()),
    onCloseTaskMenu: vi.fn(),
    onLoadMoreColumn: vi.fn(),
    onColumnScroll: vi.fn()
  }
});

describe('TaskBoard.view', () => {
  it('renders provider columns and tasks without task bodies', () => {
    const props = makeProps();
    const { queryByText, getAllByText, getByText } = render(<TaskBoardView {...props} />);

    expect(getByText('Proxy Board')).toBeTruthy();
    expect(getByText('Create proxy API')).toBeTruthy();
    expect(queryByText('Hidden task body')).toBeNull();
    expect(getAllByText('To Do').length).toBeGreaterThan(0);
    expect(getAllByText('Done').length).toBeGreaterThan(0);
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
    fireEvent.click(getByLabelText('Task actions for Create proxy API'));

    expect(props.actions.onOpenTaskMenu).toHaveBeenCalledWith(expect.any(Object), 'task-1');
  });

  it('moves a task from the right-click action menu', () => {
    const props = makeProps();
    props.data.actionMenuTaskId = 'task-1';
    const { getByText } = render(<TaskBoardView {...props} />);

    fireEvent.click(getByText('Mark Done'));

    expect(props.actions.onMoveTask).toHaveBeenCalledWith('task-1', 'done');
  });

  it('opens task details through the provided action', () => {
    const props = makeProps();
    const { getByText } = render(<TaskBoardView {...props} />);

    fireEvent.click(getByText('Create proxy API'));

    expect(props.actions.onOpenTask).toHaveBeenCalledWith(props.data.board.columns[0].tasks[0]);
  });

  it('shows task body only in the details modal', () => {
    const props = makeProps();
    props.data.selectedTask = props.data.board.columns[0].tasks[0];
    const { getByText } = render(<TaskBoardView {...props} />);

    expect(getByText('Hidden task body')).toBeTruthy();
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

  it('sets board column roles from the settings modal', () => {
    const props = makeProps();
    props.data.settingsDialogOpen = true;
    const { container } = render(<TaskBoardView {...props} />);
    const reviewColumn = container.querySelector(
      '#board-settings-review-pickup-column'
    ) as HTMLSelectElement;

    fireEvent.change(reviewColumn, { target: { value: 'done' } });

    expect(props.actions.onSetColumnRole).toHaveBeenCalledWith('done', 'review');
  });

  it('pins selected repositories and filters available repositories', () => {
    const props = makeProps();
    props.data.settingsDialogOpen = true;
    props.data.repoItems = [
      { value: 'owner/repo', label: 'owner/repo' },
      { value: 'owner/selected', label: 'owner/selected' },
      { value: 'team/api', label: 'team/api' }
    ];
    props.data.selectedRepoNames = ['owner/selected'];
    const { getByLabelText, queryByLabelText } = render(<TaskBoardView {...props} />);

    expect((getByLabelText('owner/selected') as HTMLInputElement).checked).toBe(true);

    const repoFilterInput = getByLabelText('Filter repositories') as HTMLInputElement;
    repoFilterInput.value = 'team';
    fireEvent.input(repoFilterInput);

    expect(getByLabelText('owner/selected')).toBeTruthy();
    expect(getByLabelText('team/api')).toBeTruthy();
    expect(queryByLabelText('owner/repo')).toBeNull();
  });

  it('loads more tasks from a paginated column', () => {
    const props = makeProps();
    props.data.board.columns[0].tasks = Array.from({ length: 21 }, (_, index) =>
      makeTask(`task-${index}`, `Task ${index}`)
    );
    props.data.visibleTaskCounts = { 'to-do': 20, done: 20 };
    const { getByText, queryByText } = render(<TaskBoardView {...props} />);

    expect(getByText('Task 19')).toBeTruthy();
    expect(queryByText('Task 20')).toBeNull();
    fireEvent.click(getByText('Load more'));

    expect(props.actions.onLoadMoreColumn).toHaveBeenCalledWith('to-do');
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
