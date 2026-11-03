import { QueryClient } from '@tanstack/react-query';
import { isClientError } from './errors';

export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 60_000,
      refetchOnWindowFocus: false,
      // Lỗi 4xx (404, 401…) không thử lại; lỗi mạng / 5xx thử lại một lần.
      retry: (failureCount, error) => !isClientError(error) && failureCount < 1,
    },
    mutations: { retry: false },
  },
});
