import { signal } from '@preact/signals';
import { fireEvent, render } from '@testing-library/preact';
import { describe, expect, it, vi } from 'vitest';

import { WorkersView } from '../components/workers/ui/Workers.view';

const makeWorker = (overrides = {}) => ({
  id: '1',
  name: 'test-worker',
  status: 'idle' as const,
  lastSeen: '2 minutes ago',
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

  it('renders the generate code button', () => {
    const { getByText } = render(
      <WorkersView
        data={{ workers: [], code: null, countdown }}
        status={{ loading: false, error: null, generateLoading: false, deletingId, deleteError }}
        actions={{ onGenerateCode, onConfirmDelete, onCancelDelete, onDeleteWorker }}
      />
    );

    expect(getByText('Generate Code')).toBeDefined();
  });

  it('shows the generated code and countdown', () => {
    countdown.value = '9m 30s remaining';

    const { getByText } = render(
      <WorkersView
        data={{ workers: [], code: 'a1b2c3d4e5f6g7h8', countdown }}
        status={{ loading: false, error: null, generateLoading: false, deletingId, deleteError }}
        actions={{ onGenerateCode, onConfirmDelete, onCancelDelete, onDeleteWorker }}
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
        status={{ loading: false, error: null, generateLoading: false, deletingId, deleteError }}
        actions={{ onGenerateCode, onConfirmDelete, onCancelDelete, onDeleteWorker }}
      />
    );

    expect(getByText('runner-1')).toBeDefined();
    expect(getByText('runner-2')).toBeDefined();
  });

  it('shows empty state when no workers', () => {
    const { getByText } = render(
      <WorkersView
        data={{ workers: [], code: null, countdown }}
        status={{ loading: false, error: null, generateLoading: false, deletingId, deleteError }}
        actions={{ onGenerateCode, onConfirmDelete, onCancelDelete, onDeleteWorker }}
      />
    );

    expect(getByText('No workers registered yet.')).toBeDefined();
  });

  it('shows loading text when loading', () => {
    const { getByText } = render(
      <WorkersView
        data={{ workers: [], code: null, countdown }}
        status={{ loading: true, error: null, generateLoading: false, deletingId, deleteError }}
        actions={{ onGenerateCode, onConfirmDelete, onCancelDelete, onDeleteWorker }}
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
        status={{ loading: false, error, generateLoading: false, deletingId, deleteError }}
        actions={{ onGenerateCode, onConfirmDelete, onCancelDelete, onDeleteWorker }}
      />
    );

    expect(getByText('Server error')).toBeDefined();
  });

  it('shows inline confirmation when deleting a worker', () => {
    deletingId.value = '1';

    const workers = [makeWorker({ id: '1', name: 'runner-1' })];

    const { getByText } = render(
      <WorkersView
        data={{ workers, code: null, countdown }}
        status={{ loading: false, error: null, generateLoading: false, deletingId, deleteError }}
        actions={{ onGenerateCode, onConfirmDelete, onCancelDelete, onDeleteWorker }}
      />
    );

    expect(getByText('Confirm?')).toBeDefined();
  });

  it('calls onDeleteWorker when confirm delete is clicked', () => {
    deletingId.value = '1';

    const workers = [makeWorker({ id: '1', name: 'runner-1' })];

    const { getByText } = render(
      <WorkersView
        data={{ workers, code: null, countdown }}
        status={{ loading: false, error: null, generateLoading: false, deletingId, deleteError }}
        actions={{ onGenerateCode, onConfirmDelete, onCancelDelete, onDeleteWorker }}
      />
    );

    fireEvent.click(getByText('Delete'));
    expect(onDeleteWorker).toHaveBeenCalledWith('1');
  });
});
