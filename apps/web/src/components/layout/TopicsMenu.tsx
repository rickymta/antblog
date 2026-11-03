import { useCallback, useId, useRef, useState } from 'react';
import { Link } from 'react-router';
import { useTranslation } from 'react-i18next';
import { useQuery } from '@tanstack/react-query';
import { ChevronDown } from 'lucide-react';
import { CategoryGlyph, Skeleton } from '@tkh/ui';
import { api, formatNumber, pickLocalized, type Category } from '@tkh/shared';
import { useLocale } from '../../i18n';
import { useDismiss } from '../../hooks/useDismiss';
import { safeColor, withAlpha } from '../../lib/format';

export const categoriesQuery = { queryKey: ['categories'] as const, queryFn: () => api.categories.list(), staleTime: 5 * 60_000 };

/** Danh sách chủ đề (dùng ở menu thả, ngăn kéo điện thoại, chân trang). */
export function TopicLinks({ categories, onNavigate, dense = false }: { categories: Category[]; onNavigate?: () => void; dense?: boolean }) {
  const { t } = useTranslation();
  const locale = useLocale();
  return (
    <ul className={dense ? 'space-y-0.5' : 'space-y-1'}>
      {categories.map((c) => {
        const color = safeColor(c.color);
        return (
          <li key={c.id}>
            <Link
              to={`/topics/${encodeURIComponent(c.slug)}`}
              onClick={onNavigate}
              className="flex items-center gap-3 rounded-ui px-2 py-2 text-sm text-ink hover:bg-surface-2"
            >
              <span className="inline-flex h-8 w-8 shrink-0 items-center justify-center rounded-ui" style={{ backgroundColor: withAlpha(color, 0.14) }}>
                <CategoryGlyph icon={c.icon} className="h-4 w-4" style={{ color }} />
              </span>
              <span className="min-w-0 flex-1 truncate font-medium">{pickLocalized(c.name, locale)}</span>
              <span className="shrink-0 text-xs text-muted">{t('common.articleCount', { count: c.articleCount, n: formatNumber(c.articleCount, locale) })}</span>
            </Link>
          </li>
        );
      })}
    </ul>
  );
}

/** Mục "Chủ đề ▾" trên thanh đầu (màn rộng). */
export function TopicsMenu() {
  const { t } = useTranslation();
  const [open, setOpen] = useState(false);
  const wrapRef = useRef<HTMLDivElement>(null);
  const btnRef = useRef<HTMLButtonElement>(null);
  const panelId = useId();
  const close = useCallback(() => setOpen(false), []);
  useDismiss(wrapRef, open, close, btnRef);
  const categories = useQuery({ ...categoriesQuery, enabled: open });

  return (
    <div ref={wrapRef} className="relative">
      <button
        ref={btnRef}
        type="button"
        aria-expanded={open}
        aria-controls={panelId}
        onClick={() => setOpen((v) => !v)}
        className="inline-flex items-center gap-1 rounded-ui px-3 py-2 text-sm font-medium whitespace-nowrap text-muted hover:bg-surface-2 hover:text-ink"
      >
        {t('nav.topics')}
        <ChevronDown aria-hidden="true" className={`h-4 w-4 transition-transform ${open ? 'rotate-180' : ''}`} />
      </button>
      {open && (
        <div id={panelId} className="absolute left-0 top-full z-50 mt-2 w-80 rounded-card border border-line bg-surface p-2 shadow-xl shadow-slate-900/10">
          {categories.isPending ? (
            <div className="space-y-2 p-2" aria-busy="true">
              {[0, 1, 2, 3].map((i) => (
                <Skeleton key={i} className="h-8 w-full" />
              ))}
            </div>
          ) : categories.isError ? (
            <p className="p-3 text-sm text-muted">{t('nav.topicsLoadFailed')}</p>
          ) : (
            <TopicLinks categories={categories.data} onNavigate={close} />
          )}
          <div className="mt-1 border-t border-line pt-1">
            <Link to="/articles" onClick={close} className="block rounded-ui px-2 py-2 text-sm font-semibold text-primary-ink hover:bg-surface-2">
              {t('nav.allTopics')} →
            </Link>
          </div>
        </div>
      )}
    </div>
  );
}
