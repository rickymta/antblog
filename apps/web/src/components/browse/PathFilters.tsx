'use client';

import { useEffect, useOptimistic, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { Search } from 'lucide-react';
import { LEVELS, type Level } from '@tkh/shared';
import { useDebouncedValue } from '../../hooks/useDebouncedValue';
import { inputClass } from '../../lib/ui';
import { useBrowse } from './BrowseContext';

interface PathQuery {
  level?: Level;
  q?: string;
}

function hrefOf(q: PathQuery): string {
  const p = new URLSearchParams();
  if (q.q) p.set('q', q.q);
  if (q.level) p.set('level', q.level);
  const s = p.toString();
  return s ? `/learning-paths?${s}` : '/learning-paths';
}

/** Ô tìm + nút cấp độ của /learning-paths — bộ lọc trên URL, kết quả do máy chủ render. */
export function PathFilters({ query }: { query: PathQuery }) {
  const { t } = useTranslation();
  const { navigate } = useBrowse();
  const [current, setCurrent] = useOptimistic(query);
  const urlQ = query.q ?? '';

  const [search, setSearch] = useState(urlQ);
  useEffect(() => setSearch((s) => (s.trim() === urlQ.trim() ? s : urlQ)), [urlQ]);
  const debounced = useDebouncedValue(search.trim(), 350);

  const update = (changes: PathQuery) => {
    const next: PathQuery = { ...current, ...changes };
    if (!next.q) delete next.q;
    if (!next.level) delete next.level;
    navigate(hrefOf(next), () => setCurrent(next));
  };

  useEffect(() => {
    if (debounced !== urlQ.trim()) update({ q: debounced || undefined });
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [debounced]);

  const pill = (active: boolean) =>
    `min-h-11 rounded-full px-4 py-2 text-sm font-semibold transition-colors ${active ? 'bg-primary text-white' : 'border border-line bg-surface text-ink hover:bg-surface-2'}`;

  return (
    <div className="space-y-4">
      <form
        role="search"
        action="/learning-paths"
        method="get"
        className="relative"
        onSubmit={(e) => {
          e.preventDefault();
          update({ q: search.trim() || undefined });
        }}
      >
        <label htmlFor="paths-search" className="sr-only">
          {t('paths.searchLabel')}
        </label>
        <Search aria-hidden="true" className="pointer-events-none absolute top-1/2 left-4 h-5 w-5 -translate-y-1/2 text-muted" />
        <input
          id="paths-search"
          name="q"
          type="search"
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          placeholder={t('paths.searchPlaceholder')}
          className={`${inputClass} h-12 rounded-card pl-12 text-base`}
        />
      </form>
      <div role="group" aria-label={t('articles.level')} className="flex flex-wrap gap-2">
        <button type="button" aria-pressed={!current.level} className={pill(!current.level)} onClick={() => update({ level: undefined })}>
          {t('common.allLevels')}
        </button>
        {LEVELS.map((l) => (
          <button key={l} type="button" aria-pressed={current.level === l} className={pill(current.level === l)} onClick={() => update({ level: l })}>
            {t(`common.levels.${l}`)}
          </button>
        ))}
      </div>
    </div>
  );
}
