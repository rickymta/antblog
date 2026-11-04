'use client';

import { createContext, useCallback, useContext, useMemo, type ReactNode } from 'react';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import { api, type User } from '@tkh/shared';
import { isUnauthenticated } from './errors';

export const ME_KEY = ['auth', 'me'] as const;

interface AuthState {
  user: User | null;
  /** Đang hỏi máy chủ lần đầu — chưa biết đã đăng nhập hay chưa. */
  loading: boolean;
  setUser: (user: User | null) => void;
  logout: () => Promise<void>;
}

const AuthContext = createContext<AuthState | null>(null);

/** Tải phiên từ `GET /auth/me`. 401 = chưa đăng nhập (không phải lỗi); lỗi khác (API tắt) cũng coi như khách. */
export function AuthProvider({ children }: { children: ReactNode }) {
  const qc = useQueryClient();
  const me = useQuery({
    queryKey: ME_KEY,
    queryFn: async () => {
      try {
        return (await api.auth.me()).user;
      } catch (e) {
        if (isUnauthenticated(e)) return null;
        throw e;
      }
    },
    staleTime: 5 * 60_000,
    retry: false,
  });

  const setUser = useCallback(
    (user: User | null) => {
      qc.setQueryData(ME_KEY, user);
      // Dữ liệu phụ thuộc người xem (myReactions, likedByMe) phải tải lại theo phiên mới.
      void qc.invalidateQueries({ predicate: (q) => q.queryKey[0] === 'article' || q.queryKey[0] === 'comments' });
    },
    [qc],
  );

  const logout = useCallback(async () => {
    try {
      await api.auth.logout();
    } finally {
      setUser(null);
    }
  }, [setUser]);

  const value = useMemo<AuthState>(
    () => ({ user: me.data ?? null, loading: me.isPending, setUser, logout }),
    [me.data, me.isPending, setUser, logout],
  );
  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}

export function useAuth(): AuthState {
  const ctx = useContext(AuthContext);
  if (!ctx) throw new Error('useAuth phải nằm trong <AuthProvider>');
  return ctx;
}
