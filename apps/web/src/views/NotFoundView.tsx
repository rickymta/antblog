'use client';

import Link from 'next/link';
import { useTranslation } from 'react-i18next';
import { Compass } from 'lucide-react';
import { btnPrimary, btnSecondary, container } from '../lib/ui';

type NotFoundKey = 'article.notFound' | 'topic.notFound' | 'paths.notFound';

/** Trang 404 — cũng dùng khi API trả 404 cho bài/chủ đề/lộ trình (truyền khoá câu báo riêng). */
export function NotFoundView({ messageKey }: { messageKey?: NotFoundKey }) {
  const { t } = useTranslation();
  const message = messageKey ? t(messageKey) : undefined;
  return (
    <div className={`${container} flex flex-col items-center py-24 text-center`}>
      <span className="inline-flex h-16 w-16 items-center justify-center rounded-full bg-primary-soft text-primary-ink">
        <Compass aria-hidden="true" className="h-8 w-8" />
      </span>
      <p className="mt-6 font-mono text-sm font-semibold text-primary-ink">404</p>
      <h1 className="mt-2 text-3xl font-extrabold tracking-tight text-ink sm:text-4xl">{t('notFound.title')}</h1>
      <p className="mt-3 max-w-md text-muted">{message ?? t('notFound.body')}</p>
      <div className="mt-8 flex flex-wrap justify-center gap-3">
        <Link href="/" className={btnPrimary}>
          {t('common.backHome')}
        </Link>
        <Link href="/articles" className={btnSecondary}>
          {t('notFound.articles')}
        </Link>
      </div>
    </div>
  );
}
