'use client';

import { useState, type ReactNode } from 'react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import type { Locale } from '@tkh/shared';
import { I18nProvider } from '../i18n';
import { AuthProvider } from '../lib/auth';
import { isClientError } from '../lib/errors';
import { useVisitTracking } from '../hooks/useVisitTracking';

function makeQueryClient() {
  return new QueryClient({
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
}

function VisitTracker() {
  useVisitTracking();
  return null;
}

/**
 * Ngữ cảnh phía trình duyệt: React Query (phiên đăng nhập, bình luận, reaction…), i18next, phiên người dùng.
 * QueryClient tạo trong state ⇒ mỗi request máy chủ một bản, trình duyệt một bản cho cả phiên.
 */
export function Providers({ locale, children }: { locale: Locale; children: ReactNode }) {
  const [queryClient] = useState(makeQueryClient);
  return (
    <QueryClientProvider client={queryClient}>
      <I18nProvider locale={locale}>
        <AuthProvider>
          <VisitTracker />
          {children}
        </AuthProvider>
      </I18nProvider>
    </QueryClientProvider>
  );
}
