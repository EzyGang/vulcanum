import { signal } from '@preact/signals';
import { fireEvent, render } from '@testing-library/preact';
import { describe, expect, it, vi } from 'vitest';
import { WorkersView } from '../components/workers/ui/Workers.view';
import type { ApiError } from '../utils/api/client';

const makeWorker = (overrides = {}) => ({
  id: '1',
  name: 'test-worker',
  status: 'idle' as const,
  lastSeen: '2 minutes ago',
  activeJobs: 0,
  maxConcurrentJobs: 3,
  consecutiveErrors: 0,
  ...overrides
});

describe('Workers.view', () => {
  const countdown = signal('');
  const deletingId = signal<string | null>(null);
  const deleteError = signal<string | null>(null);
  const onGenerateCode = vi.fn();
  const onConfirmDelete = vi.fn();
  const onCancelDelete = vi.fn();
  const onDeleteWorker = vi.fn();
  const onUpdateStatus = vi.fn();

  const baseStatus = {
    loading: false,
    error: null as ApiError | null,
    generateLoading: false,
    deletingId,
    deleteError,
    updateStatusError: null as ApiError | null
  };
  const baseActions = {
    onGenerateCode,
    onConfirmDelete,
    onCancelDelete,
    onDeleteWorker,
    onUpdateStatus
  };

  it('renders the generate code button', () => {
    const { getByText } = render(
      <WorkersView
        data={{ workers: [], code: null, countdown }}
        status={baseStatus}
        actions={baseActions}
      />
    );

    expect(getByText('Generate Code')).toBeDefined();
  });

  it('shows the generated code and countdown', () => {
    countdown.value = '9m 30s remaining';

    const { getByText } = render(
      <WorkersView
        data={{ workers: [], code: 'a1b2c3d4e5f6g7h8', countdown }}
        status={baseStatus}
        actions={baseActions}
      />
    );

    expect(getByText('a1b2c3d4e5f6g7h8')).toBeDefined();
    expect(getByText('9m 30s remaining')).toBeDefined();
  });

  it('renders workers in the table', () => {
    const workers = [
      makeWorker({ id: '1', name: 'runner-1' }),
      makeWorker({ id: '2', name: 'runner-2', status: 'busy' as const })
    ];

    const { getByText } = render(
      <WorkersView
        data={{ workers, code: null, countdown }}
        status={baseStatus}
        actions={baseActions}
      />
    );

    expect(getByText('runner-1')).toBeDefined();
    expect(getByText('runner-2')).toBeDefined();
  });

  it('shows empty state when no workers', () => {
    const { getByText } = render(
      <WorkersView
        data={{ workers: [], code: null, countdown }}
        status={baseStatus}
        actions={baseActions}
      />
    );

    expect(getByText('No workers registered yet.')).toBeDefined();
  });

  it('shows loading text when loading', () => {
    const { getByText } = render(
      <WorkersView
        data={{ workers: [], code: null, countdown }}
        status={{ ...baseStatus, loading: true }}
        actions={baseActions}
      />
    );

    expect(getByText('Loading workers...')).toBeDefined();
  });

  it('shows error message when error is set', () => {
    const error = {
      name: 'ApiError',
      message: 'Server error',
      status: 500,
      serverError: 'Server error'
    };

    const { getByText } = render(
      <WorkersView
        data={{ workers: [], code: null, countdown }}
        status={{ ...baseStatus, error }}
        actions={baseActions}
      />
    );

    expect(getByText('Server error')).toBeDefined();
  });

  it('shows inline confirmation when deleting a worker', () => {
    deletingId.value = '1';

    const workers = [makeWorker({ id: '1', name: 'runner-1' })];

    const { getByLabelText } = render(
      <WorkersView
        data={{ workers, code: null, countdown }}
        status={baseStatus}
        actions={baseActions}
      />
    );

    expect(getByLabelText('Confirm delete')).toBeDefined();
  });

  it('calls onDeleteWorker when confirm delete is clicked', () => {
    deletingId.value = '1';

    const workers = [makeWorker({ id: '1', name: 'runner-1' })];

    const { getByLabelText } = render(
      <WorkersView
        data={{ workers, code: null, countdown }}
        status={baseStatus}
        actions={baseActions}
      />
    );

    fireEvent.click(getByLabelText('Confirm delete'));
    expect(onDeleteWorker).toHaveBeenCalledWith('1');
  });

  it('renders load column with correct fraction', () => {
    const workers = [
      makeWorker({ id: '1', name: 'runner-1', activeJobs: 2, maxConcurrentJobs: 5 })
    ];

    const { getByText } = render(
      <WorkersView
        data={{ workers, code: null, countdown }}
        status={baseStatus}
        actions={baseActions}
      />
    );

    expect(getByText('2 / 5')).toBeDefined();
  });

  it('shows red progress bar when worker is at max capacity', () => {
    const workers = [
      makeWorker({ id: '1', name: 'runner-1', activeJobs: 3, maxConcurrentJobs: 3 })
    ];

    const { container } = render(
      <WorkersView
        data={{ workers, code: null, countdown }}
        status={baseStatus}
        actions={baseActions}
      />
    );

    const fill = container.querySelector('.bg-error');
    expect(fill).toBeDefined();
  });

  it('shows Disable action for idle workers', () => {
    const workers = [makeWorker({ id: '1', name: 'worker-1', status: 'idle' as const })];

    const { getByLabelText, queryByText } = render(
      <WorkersView
        data={{ workers, code: null, countdown }}
        status={baseStatus}
        actions={baseActions}
      />
    );

    expect(getByLabelText('Disable worker')).toBeDefined();
    expect(queryByText('Disable')).toBeNull();
  });

  it('shows Re-enable action for unhealthy workers', () => {
    const workers = [makeWorker({ id: '1', name: 'worker-1', status: 'unhealthy' as const })];

    const { getByLabelText, queryByText } = render(
      <WorkersView
        data={{ workers, code: null, countdown }}
        status={baseStatus}
        actions={baseActions}
      />
    );

    expect(getByLabelText('Re-enable worker')).toBeDefined();
    expect(queryByText('Re-enable')).toBeNull();
  });
});
