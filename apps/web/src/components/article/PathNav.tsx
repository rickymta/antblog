import { useId, useState } from 'react';
import { Link } from 'react-router';
import { useTranslation } from 'react-i18next';
import { Check, ChevronRight } from 'lucide-react';
import type { ArticleLearningPathRef } from '@tkh/shared';
import { useReadArticles } from '../../lib/reading';
import { articleHref } from '../common/ArticleCard';
import { LevelTag } from '../common/Badges';
import { pathHref } from '../common/PathCard';

/**
 * Danh sách bài của lộ trình (A4): bài đang đọc nền primary nhạt chữ primary, bài đã đọc có ✓.
 * Nhóm thu/mở bằng mũi tên `›`; mặc định mở.
 */
export function PathNav({ path, currentSlug, defaultOpen = true }: { path: ArticleLearningPathRef; currentSlug: string; defaultOpen?: boolean }) {
  const { t } = useTranslation();
  const read = useReadArticles();
  const [open, setOpen] = useState(defaultOpen);
  const listId = useId();

  return (
    <nav aria-label={t('article.pathNav')}>
      <p className="mb-1 text-xs font-bold tracking-wide text-muted uppercase">{t('article.inPath')}</p>
      <div className="flex items-start gap-1">
        <button
          type="button"
          onClick={() => setOpen((v) => !v)}
          aria-expanded={open}
          aria-controls={listId}
          aria-label={t('article.pathNav')}
          className="mt-0.5 inline-flex h-6 w-6 shrink-0 items-center justify-center rounded text-muted hover:bg-surface-2 hover:text-ink"
        >
          <ChevronRight aria-hidden="true" className={`h-4 w-4 transition-transform ${open ? 'rotate-90' : ''}`} />
        </button>
        <div className="min-w-0">
          <Link to={pathHref(path.slug)} className="font-semibold leading-snug text-ink hover:text-primary-ink hover:underline">
            {path.title}
          </Link>
          <div className="mt-1">
            <LevelTag level={path.level} />
          </div>
        </div>
      </div>
      {open && (
        <ol id={listId} className="mt-3 space-y-0.5">
          {path.articles.map((a, i) => {
            const current = a.slug === currentSlug;
            const done = Boolean(read[a.id]);
            return (
              <li key={a.id}>
                <Link
                  to={articleHref(a.slug)}
                  aria-current={current ? 'page' : undefined}
                  className={`flex items-start gap-2.5 rounded-ui px-2 py-2 text-sm leading-snug transition-colors ${
                    current ? 'bg-primary-soft font-semibold text-primary-ink' : 'text-ink hover:bg-surface-2'
                  }`}
                >
                  <span
                    aria-hidden="true"
                    className={`mt-px inline-flex h-5 w-5 shrink-0 items-center justify-center rounded-full text-[11px] font-bold ${
                      done && !current ? 'bg-emerald-600 text-white' : current ? 'bg-primary text-white' : 'bg-surface-2 text-ink dark:bg-slate-700'
                    }`}
                  >
                    {done && !current ? <Check className="h-3 w-3" /> : i + 1}
                  </span>
                  <span className="min-w-0">
                    {a.title}
                    {current && <span className="sr-only"> ({t('article.current')})</span>}
                    {done && !current && <span className="sr-only"> ({t('paths.done')})</span>}
                  </span>
                </Link>
              </li>
            );
          })}
        </ol>
      )}
    </nav>
  );
}
