import { QueryClient } from '@tanstack/react-query';

export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: 0,
      staleTime: 60_000,
      gcTime: 5 * 60_000
    }
  }
});

export const invalidate = (...keyParts: unknown[]) =>
  queryClient.invalidateQueries({ queryKey: keyParts, refetchType: 'all' });
