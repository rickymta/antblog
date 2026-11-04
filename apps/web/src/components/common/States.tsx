'use client';

import { useTransition, type ReactNode } from 'react';
import { useRouter } from 'next/navigation';
import { useTranslation } from 'react-i18next';
import { Inbox, RefreshCw, TriangleAlert } from 'lucide-react';
import { btnSecondary } from '../../lib/ui';

/** Khối báo lỗi tải dữ liệu, có nút thử lại. Dùng `role="alert"` để trình đọc màn hình đọc ngay. */
export function ErrorState({
  onRetry,
  title,
  message,
  compact = false,
}: {
  onRetry?: () => void;
  title?: string;
  message?: string;
  compact?: boolean;
}) {
  const { t } = useTranslation();
  return (
    <div
      role="alert"
      className={`flex flex-col items-center gap-3 rounded-card border border-dashed border-line bg-surface text-center ${compact ? 'p-4' : 'px-6 py-10'}`}
    >
      <TriangleAlert aria-hidden="true" className="h-7 w-7 text-amber-600 dark:text-amber-400" />
      <div>
        <p className="font-semibold text-ink">{title ?? t('common.errorTitle')}</p>
        <p className="mt-1 text-sm text-muted">{message ?? t('common.errorBody')}</p>
      </div>
      {onRetry && (
        <button type="button" onClick={onRetry} className={btnSecondary}>
          <RefreshCw aria-hidden="true" className="h-4 w-4" />
          {t('common.retry')}
        </button>
      )}
    </div>
  );
}

export function EmptyState({ title, hint, action, icon }: { title: string; hint?: string; action?: ReactNode; icon?: ReactNode }) {
  return (
    <div className="flex flex-col items-center gap-3 rounded-card border border-dashed border-line bg-surface px-6 py-10 text-center">
      {icon ?? <Inbox aria-hidden="true" className="h-7 w-7 text-muted" />}
      <div>
        <p className="font-semibold text-ink">{title}</p>
        {hint && <p className="mt-1 text-sm text-muted">{hint}</p>}
      </div>
      {action}
    </div>
  );
}

/** Vùng đang tải: báo cho trình đọc màn hình một lần, phần khung xương ẩn với họ. */
export function LoadingRegion({ children, className = '' }: { children: ReactNode; className?: string }) {
  const { t } = useTranslation();
  return (
    <div aria-busy="true" className={className}>
      <span className="sr-only" role="status">
        {t('common.loading')}
      </span>
      {children}
    </div>
  );
}

/** Chỗ giữ khi đang tải mã của một trang (tách gói). */
export function PageLoading() {
  const { t } = useTranslation();
  return (
    <div className="flex min-h-[50vh] items-center justify-center" role="status">
      <span className="h-8 w-8 animate-spin rounded-full border-4 border-primary/25 border-t-primary motion-reduce:animate-none" aria-hidden="true" />
      <span className="sr-only">{t('common.loading')}</span>
    </div>
  );
}

/** Khối lỗi cho dữ liệu do máy chủ tải: "Thử lại" = render lại trang phía máy chủ (giữ nguyên trạng thái phía trình duyệt). */
export function RefreshErrorState({ compact = false }: { compact?: boolean }) {
  const router = useRouter();
  const [pending, startTransition] = useTransition();
  return (
    <div aria-busy={pending} className={pending ? 'opacity-60' : undefined}>
      <ErrorState compact={compact} onRetry={() => startTransition(() => router.refresh())} />
    </div>
  );
}
