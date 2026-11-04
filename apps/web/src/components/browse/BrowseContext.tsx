'use client';

import { createContext, useCallback, useContext, useTransition, type ReactNode } from 'react';
import { useRouter } from 'next/navigation';

interface BrowseState {
  pending: boolean;
  /** Đổi query string (thay mục lịch sử, không cuộn) — máy chủ render lại danh sách theo bộ lọc mới. */
  navigate: (href: string, beforeNavigate?: () => void) => void;
}

const BrowseContext = createContext<BrowseState | null>(null);

/** Bao thanh lọc + kết quả: thanh lọc gọi `navigate`, khối kết quả mờ đi trong lúc chờ máy chủ. */
export function BrowseProvider({ children }: { children: ReactNode }) {
  const router = useRouter();
  const [pending, startTransition] = useTransition();
  const navigate = useCallback(
    (href: string, beforeNavigate?: () => void) =>
      startTransition(() => {
        beforeNavigate?.();
        router.replace(href, { scroll: false });
      }),
    [router],
  );
  return <BrowseContext.Provider value={{ pending, navigate }}>{children}</BrowseContext.Provider>;
}

export function useBrowse(): BrowseState {
  const ctx = useContext(BrowseContext);
  if (!ctx) throw new Error('useBrowse phải nằm trong <BrowseProvider>');
  return ctx;
}

/** Vùng kết quả do máy chủ render; mờ đi khi đang tải bộ lọc mới. */
export function BrowseResults({ children, className = '' }: { children: ReactNode; className?: string }) {
  const { pending } = useBrowse();
  return (
    <div aria-busy={pending} className={`${className} ${pending ? 'opacity-60 transition-opacity' : ''}`}>
      {children}
    </div>
  );
}
