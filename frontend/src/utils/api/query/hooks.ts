import {
  type MutationOptions,
  type QueryKey,
  type UseQueryResult,
  useMutation,
  useQuery
} from '@tanstack/react-query';
import type { ApiError } from '../client';

type ApiQueryOptions<TData> = Omit<
  Parameters<typeof useQuery<TData, ApiError>>[0],
  'queryKey' | 'queryFn'
>;

export const useApiQuery = <TData>(
  key: QueryKey,
  fn: () => Promise<TData>,
  opts?: ApiQueryOptions<TData>
): UseQueryResult<TData, ApiError> => {
  return useQuery<TData, ApiError>({
    queryKey: key,
    queryFn: fn,
    ...opts
  });
};

export const useApiMutation = <I, O>(
  fn: (input: I) => Promise<O>,
  opts?: Omit<MutationOptions<O, ApiError, I, unknown>, 'mutationFn'>
) => {
  return useMutation<O, ApiError, I>({
    mutationFn: fn,
    ...opts
  });
};
