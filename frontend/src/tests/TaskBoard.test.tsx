import { fireEvent, render } from '@testing-library/preact';
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
      onChange={(event) => onValueChange((event.target as HTMLSelectElement).value)}
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

import type { ComponentChildren } from 'preact';

import { TaskBoardView } from '../components/task-board/ui/TaskBoard.view';

const makeProps = () => ({
  data: {
    selectedProjectKey: 'provider-1:project-1',
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
          tasks: [
            {
              id: 'task-1',
              title: 'Create proxy API',
              projectId: 'project-1',
              description: 'Expose task creation',
              status: 'to-do',
              priority: 'low',
              number: 12,
              projectSlug: 'proxy-board',
              assigneeName: null,
              createdAt: '2026-01-01T00:00:00Z',
              updatedAt: null
            }
          ]
        },
        { id: 'column-2', name: 'Done', slug: 'done', isFinal: true, tasks: [] }
      ]
    },
    repoItems: [{ value: 'owner/repo', label: 'owner/repo' }],
    selectedRepoNames: [],
    selectedTask: null
  },
  form: {
    title: '',
    body: '',
    status: 'to-do',
    createError: null,
    serverError: null
  },
  status: {
    loading: false,
    error: null,
    creating: false,
    movingTaskId: null,
    moving: false,
    reposLoading: false,
    connectingRepos: false,
    connected: false
  },
  actions: {
    onTitleInput: vi.fn(),
    onBodyInput: vi.fn(),
    onStatusChange: vi.fn(),
    onSubmitTask: vi.fn((event: Event) => event.preventDefault()),
    onMoveTask: vi.fn(),
    onToggleRepo: vi.fn(),
    onOpenTask: vi.fn(),
    onCloseTask: vi.fn(),
    onDragStart: vi.fn(),
    onDragOver: vi.fn((event: DragEvent) => event.preventDefault()),
    onDropOnStatus: vi.fn()
  }
});

describe('TaskBoard.view', () => {
  it('renders provider columns and tasks', () => {
    const props = makeProps();
    const { getAllByText, getByText } = render(<TaskBoardView {...props} />);

    expect(getByText('Proxy Board')).toBeTruthy();
    expect(getByText('Create proxy API')).toBeTruthy();
    expect(getAllByText('To Do').length).toBeGreaterThan(0);
    expect(getAllByText('Done').length).toBeGreaterThan(0);
  });

  it('submits task creation through the provided action', () => {
    const props = makeProps();
    const { getByText } = render(<TaskBoardView {...props} />);

    fireEvent.click(getByText('Create task'));

    expect(props.actions.onSubmitTask).toHaveBeenCalledOnce();
  });

  it('moves a task to another status through the provided action', () => {
    const props = makeProps();
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

  it('connects a repository through the provided action', () => {
    const props = makeProps();
    const { getByLabelText } = render(<TaskBoardView {...props} />);

    fireEvent.click(getByLabelText('owner/repo'));

    expect(props.actions.onToggleRepo).toHaveBeenCalledWith('owner/repo');
  });

  it('drops a task onto a column through the provided action', () => {
    const props = makeProps();
    const { getAllByText } = render(<TaskBoardView {...props} />);
    const doneColumn = getAllByText('Done')[1].closest('section');

    fireEvent.drop(doneColumn as Element);

    expect(props.actions.onDropOnStatus).toHaveBeenCalledWith(expect.any(Object), 'done');
  });
});
