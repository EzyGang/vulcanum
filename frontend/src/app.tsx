import { QueryClientProvider } from '@tanstack/react-query';
import type { JSX } from 'preact';
import { useAuthRefresh } from './hooks/useAuthRefresh.hook';
import { AppRouter } from './routes/AppRouter';
import { queryClient } from './utils/api/query/client';

export const App = (): JSX.Element => {
  useAuthRefresh();

  return (
    <QueryClientProvider client={queryClient}>
      <AppRouter />
    </QueryClientProvider>
  );
};
