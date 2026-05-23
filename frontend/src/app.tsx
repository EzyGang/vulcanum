import { QueryClientProvider } from '@tanstack/react-query';
import type { JSX } from 'preact';
import { AppRouter } from './routes/AppRouter';
import { queryClient } from './utils/api/query/client';

export const App = (): JSX.Element => (
  <QueryClientProvider client={queryClient}>
    <AppRouter />
  </QueryClientProvider>
);
