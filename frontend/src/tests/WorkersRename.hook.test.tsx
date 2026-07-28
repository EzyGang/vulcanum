import { fireEvent, render, waitFor } from '@testing-library/preact';
import { describe, expect, it, vi } from 'vitest';

const { renameWorker, invalidate } = vi.hoisted(() => ({
  renameWorker: vi.fn(() => Promise.resolve({})),
  invalidate: vi.fn()
}));

vi.mock('../services/workers/workers.service', () => ({
  deleteWorker: vi.fn(),
  generateCode: vi.fn(),
  listWorkers: vi.fn(),
  renameWorker,
  updateWorkerStatus: vi.fn()
}));

vi.mock('../utils/api/query/client', () => ({ invalidate }));

vi.mock('../utils/api/query/hooks', () => ({
  useApiQuery: () => ({ data: [], isLoading: false, error: null }),
  useApiMutation: (
    fn: (input: unknown) => Promise<unknown>,
    options?: { onSuccess?: () => void }
  ) => ({
    data: undefined,
    error: null,
    isPending: false,
    mutate: (input: unknown) => {
      void fn(input).then(() => options?.onSuccess?.());
    },
    mutateAsync: fn
  })
}));

import { useWorkers } from '../components/workers/hooks/useWorkers.hook';

const RenameWorkerButton = () => {
  const { handleRenameWorker } = useWorkers();

  return (
    <button type='button' onClick={() => handleRenameWorker('worker-1', 'release-runner')}>
      Rename
    </button>
  );
};

describe('useWorkers rename', () => {
  it('refreshes workers after a successful rename', async () => {
    const { getByText } = render(<RenameWorkerButton />);

    fireEvent.click(getByText('Rename'));

    await waitFor(() => {
      expect(renameWorker).toHaveBeenCalledWith('worker-1', { name: 'release-runner' });
      expect(invalidate).toHaveBeenCalledWith('workers');
    });
  });
});
