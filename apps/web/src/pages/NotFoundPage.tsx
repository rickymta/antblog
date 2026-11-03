import { Link } from 'react-router';
import { useTranslation } from 'react-i18next';
import { Compass } from 'lucide-react';
import { useDocumentTitle } from '../hooks/useDocumentTitle';
import { btnPrimary, btnSecondary, container } from '../lib/ui';

/** Trang 404 — cũng dùng khi API trả 404 cho bài/chủ đề/lộ trình (truyền `message` riêng). */
export function NotFoundPage({ message }: { message?: string }) {
  const { t } = useTranslation();
  useDocumentTitle(t('notFound.title'));
  return (
    <div className={`${container} flex flex-col items-center py-24 text-center`}>
      <span className="inline-flex h-16 w-16 items-center justify-center rounded-full bg-primary-soft text-primary-ink">
        <Compass aria-hidden="true" className="h-8 w-8" />
      </span>
      <p className="mt-6 font-mono text-sm font-semibold text-primary-ink">404</p>
      <h1 className="mt-2 text-3xl font-extrabold tracking-tight text-ink sm:text-4xl">{t('notFound.title')}</h1>
      <p className="mt-3 max-w-md text-muted">{message ?? t('notFound.body')}</p>
      <div className="mt-8 flex flex-wrap justify-center gap-3">
        <Link to="/" className={btnPrimary}>
          {t('common.backHome')}
        </Link>
        <Link to="/articles" className={btnSecondary}>
          {t('notFound.articles')}
        </Link>
      </div>
    </div>
  );
}
