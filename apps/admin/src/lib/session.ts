import { MutationCache, QueryCache, QueryClient, useQuery, useQueryClient } from '@tanstack/react-query';
import { api, type User } from '@tkh/shared';
import { isApiError, isSessionLost } from './errors';

export const SESSION_KEY = ['session'] as const;

/** Phiên hiện tại: User khi đã đăng nhập, null khi chưa (401). Lỗi khác (API không chạy) ⇒ ném để hiện màn lỗi. */
async function fetchSession(): Promise<User | null> {
  try {
    const { user } = await api.auth.me();
    return user;
  } catch (err) {
    if (isApiError(err) && err.status === 401) return null;
    throw err;
  }
}

/** Đánh dấu phiên mất ⇒ cổng đăng nhập hiện lại; dữ liệu quản trị cũ bị bỏ. */
function dropSession(client: QueryClient): void {
  if (client.getQueryData(SESSION_KEY) === null) return;
  client.setQueryData(SESSION_KEY, null);
  client.removeQueries({ predicate: (q) => q.queryKey[0] !== SESSION_KEY[0] });
}

export function createQueryClient(): QueryClient {
  const client: QueryClient = new QueryClient({
    queryCache: new QueryCache({
      onError: (err) => {
        if (isSessionLost(err)) dropSession(client);
      },
    }),
    mutationCache: new MutationCache({
      onError: (err) => {
        if (isSessionLost(err)) dropSession(client);
      },
    }),
    defaultOptions: {
      queries: {
        staleTime: 30_000,
        refetchOnWindowFocus: false,
        // 4xx là trả lời dứt khoát — không thử lại; lỗi mạng/5xx thử lại một lần.
        retry: (count, err) => (isApiError(err) && err.status < 500 ? false : count < 1),
      },
    },
  });
  return client;
}

export function useSession() {
  return useQuery({ queryKey: SESSION_KEY, queryFn: fetchSession, staleTime: 5 * 60_000, retry: false });
}

/** Người dùng đã đăng nhập (chỉ gọi bên trong khung đã qua cổng đăng nhập). */
export function useCurrentUser(): User {
  const { data } = useSession();
  if (!data) throw new Error('useCurrentUser gọi ngoài vùng đã đăng nhập');
  return data;
}

export function useIsAdmin(): boolean {
  return useCurrentUser().role === 'admin';
}

export function useLogout(): () => Promise<void> {
  const client = useQueryClient();
  return async () => {
    try {
      await api.auth.logout();
    } catch {
      /* API lỗi vẫn bỏ phiên phía giao diện */
    }
    client.setQueryData(SESSION_KEY, null);
    client.removeQueries({ predicate: (q) => q.queryKey[0] !== SESSION_KEY[0] });
  };
}
