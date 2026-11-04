'use client';

import { useState } from 'react';
import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { useTranslation } from 'react-i18next';
import { LogIn } from 'lucide-react';
import { formatCompact, type ReactionType } from '@tkh/shared';
import { useLocale } from '../../i18n';
import { useAuth } from '../../lib/auth';
import { loginHref } from '../../lib/navigation';
import { useArticle } from './ArticleContext';
import { REACTIONS } from './reactionMeta';
import { useReactions } from './useReactions';

/**
 * "Bạn thấy bài viết này thế nào?" — 4 thẻ reaction ngang bằng nhau (A7): emoji lớn, tên tiếng Anh đậm, nghĩa tiếng Việt nhỏ, số đếm.
 * Đã chọn ⇒ viền primary + nền primary nhạt; bấm lại để bỏ. Chọn được nhiều loại. Số đếm có sẵn trong HTML máy chủ.
 */
export function ReactionCards() {
  const { t } = useTranslation();
  const locale = useLocale();
  const pathname = usePathname();
  const article = useArticle();
  const { loading } = useAuth();
  const { toggle, pending, loggedIn } = useReactions(article.slug);
  const [message, setMessage] = useState<'login' | 'error' | null>(null);

  const onClick = async (type: ReactionType) => {
    const outcome = await toggle(type);
    setMessage(outcome === 'login' ? 'login' : outcome === 'error' ? 'error' : null);
  };

  return (
    <section id="reactions" aria-labelledby="reactions-title" className="scroll-mt-24 rounded-card border border-line bg-surface p-5 sm:p-6">
      <h2 id="reactions-title" className="text-center text-xl font-bold text-ink">
        {t('article.reactionsQuestion')}
      </h2>
      <p className="mt-1 text-center text-sm text-muted">{t('article.reactionsHint')}</p>
      <ul className="mt-5 grid grid-cols-2 gap-3 md:grid-cols-4">
        {REACTIONS.map((r) => {
          const selected = article.myReactions.includes(r.type);
          const count = article.stats.reactions[r.type];
          return (
            <li key={r.type}>
              <button
                type="button"
                aria-pressed={selected}
                aria-busy={pending.has(r.type)}
                // Không đặt aria-label: tên truy cập là chính chữ trên thẻ (tên · nghĩa · số đếm), trạng thái qua aria-pressed.
                onClick={() => void onClick(r.type)}
                className={`flex h-full w-full flex-col items-center gap-1 rounded-card border-2 px-3 py-4 text-center transition-all motion-safe:active:scale-95 ${
                  selected ? 'border-primary bg-primary-soft' : 'border-line bg-surface hover:border-primary/50 hover:bg-surface-2'
                }`}
              >
                <span aria-hidden="true" className="text-3xl leading-none sm:text-4xl">
                  {r.emoji}
                </span>
                <span className="mt-1 text-sm font-bold text-ink" lang="en">
                  {r.name}
                </span>
                <span className="text-xs text-muted" lang="vi">
                  {t(`reactions.${r.type}`)}
                </span>
                <span className={`mt-1 text-sm font-semibold ${selected ? 'text-primary-ink' : 'text-ink'}`}>{formatCompact(count, locale)}</span>
              </button>
            </li>
          );
        })}
      </ul>
      <div aria-live="polite" className="mt-4 text-center text-sm">
        {(message === 'login' || (!loggedIn && !loading && message !== 'error')) && (
          <p className="inline-flex flex-wrap items-center justify-center gap-2 text-muted">
            {t('article.reactionsLogin')}
            <Link href={loginHref(pathname)} rel="nofollow" className="inline-flex items-center gap-1 font-semibold text-primary-ink hover:underline">
              <LogIn aria-hidden="true" className="h-4 w-4" />
              {t('header.login')}
            </Link>
          </p>
        )}
        {message === 'error' && <p className="font-medium text-red-700 dark:text-red-400">{t('article.reactionError')}</p>}
      </div>
    </section>
  );
}
