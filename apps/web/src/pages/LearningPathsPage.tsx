import { useEffect, useState } from 'react';
import { useSearchParams } from 'react-router';
import { useTranslation } from 'react-i18next';
import { keepPreviousData, useQuery } from '@tanstack/react-query';
import { Search, SearchX } from 'lucide-react';
import { api, LEVELS, type Level } from '@tkh/shared';
import { PageHeader } from '../components/common/PageHeader';
import { PathCard, PathCardSkeleton } from '../components/common/PathCard';
import { EmptyState, ErrorState, LoadingRegion } from '../components/common/States';
import { useDebouncedValue } from '../hooks/useDebouncedValue';
import { useDocumentTitle } from '../hooks/useDocumentTitle';
import { container, inputClass } from '../lib/ui';

export function LearningPathsPage() {
  const { t } = useTranslation();
  useDocumentTitle(t('nav.paths'));
  const [params, setParams] = useSearchParams();
  const levelParam = params.get('level');
  const level = LEVELS.includes(levelParam as Level) ? (levelParam as Level) : undefined;
  const urlQ = params.get('q') ?? '';

  const [search, setSearch] = useState(urlQ);
  useEffect(() => setSearch((s) => (s.trim() === urlQ.trim() ? s : urlQ)), [urlQ]);
  const debounced = useDebouncedValue(search.trim(), 350);

  const setParam = (key: string, value: string | undefined) =>
    setParams(
      (prev) => {
        const next = new URLSearchParams(prev);
        if (value) next.set(key, value);
        else next.delete(key);
        return next;
      },
      { replace: true, preventScrollReset: true },
    );

  useEffect(() => {
    if (debounced !== urlQ.trim()) setParam('q', debounced || undefined);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [debounced]);

  const q = urlQ.trim() || undefined;
  const paths = useQuery({
    queryKey: ['learning-paths', level ?? '', q ?? ''],
    queryFn: () => api.learningPaths.list({ level, q }),
    placeholderData: keepPreviousData,
  });

  const pill = (active: boolean) =>
    `rounded-full px-4 py-1.5 text-sm font-semibold transition-colors ${active ? 'bg-primary text-white' : 'border border-line bg-surface text-ink hover:bg-surface-2'}`;

  return (
    <div className={container}>
      <PageHeader title={t('paths.title')} subtitle={t('paths.subtitle')} />
      <div className="space-y-4">
        <form
          role="search"
          className="relative"
          onSubmit={(e) => {
            e.preventDefault();
            setParam('q', search.trim() || undefined);
          }}
        >
          <label htmlFor="paths-search" className="sr-only">
            {t('paths.searchLabel')}
          </label>
          <Search aria-hidden="true" className="pointer-events-none absolute top-1/2 left-4 h-5 w-5 -translate-y-1/2 text-muted" />
          <input
            id="paths-search"
            type="search"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder={t('paths.searchPlaceholder')}
            className={`${inputClass} h-12 rounded-card pl-12 text-base`}
          />
        </form>
        <div role="group" aria-label={t('articles.level')} className="flex flex-wrap gap-2">
          <button type="button" aria-pressed={!level} className={pill(!level)} onClick={() => setParam('level', undefined)}>
            {t('common.allLevels')}
          </button>
          {LEVELS.map((l) => (
            <button key={l} type="button" aria-pressed={level === l} className={pill(level === l)} onClick={() => setParam('level', l)}>
              {t(`common.levels.${l}`)}
            </button>
          ))}
        </div>
      </div>

      <div className="mt-8">
        {paths.isPending ? (
          <LoadingRegion className="grid gap-5 sm:grid-cols-2 xl:grid-cols-4">
            {[0, 1, 2, 3].map((i) => (
              <PathCardSkeleton key={i} />
            ))}
          </LoadingRegion>
        ) : paths.isError ? (
          <ErrorState onRetry={() => void paths.refetch()} />
        ) : paths.data.length === 0 ? (
          <EmptyState icon={<SearchX aria-hidden="true" className="h-7 w-7 text-muted" />} title={t('paths.empty')} />
        ) : (
          <ul className={`grid gap-5 sm:grid-cols-2 xl:grid-cols-4 ${paths.isPlaceholderData ? 'opacity-60' : ''}`}>
            {paths.data.map((p) => (
              <li key={p.id}>
                <PathCard path={p} showDate />
              </li>
            ))}
          </ul>
        )}
      </div>
    </div>
  );
}
