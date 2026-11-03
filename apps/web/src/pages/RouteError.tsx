import { useTranslation } from 'react-i18next';
import { TriangleAlert } from 'lucide-react';

/**
 * Lưới an toàn cuối cùng khi một trang ném lỗi lúc render: không để màn trắng.
 * Không phụ thuộc router/Query (có thể chính chúng đang lỗi) — tải lại trang là cách phục hồi.
 */
export function RouteError() {
  const { t } = useTranslation();
  return (
    <div className="flex min-h-dvh flex-col items-center justify-center gap-4 bg-page px-6 text-center">
      <TriangleAlert aria-hidden="true" className="h-10 w-10 text-amber-600 dark:text-amber-400" />
      <h1 className="text-2xl font-bold text-ink">{t('common.errorTitle')}</h1>
      <p className="max-w-md text-muted">{t('common.errorBody')}</p>
      <div className="flex gap-3">
        <button
          type="button"
          onClick={() => window.location.reload()}
          className="rounded-ui bg-primary px-4 py-2 text-sm font-semibold text-white hover:bg-primary-hover"
        >
          {t('common.retry')}
        </button>
        <a href="/" className="rounded-ui border border-line bg-surface px-4 py-2 text-sm font-semibold text-ink hover:bg-surface-2">
          {t('common.backHome')}
        </a>
      </div>
    </div>
  );
}
