'use client';

import { useEffect, useId, useOptimistic, useState, type FormEvent, type ReactNode } from 'react';
import { useTranslation } from 'react-i18next';
import { Search, X } from 'lucide-react';
import { LEVELS, categoryOptions, pickLocalized, type Category } from '@tkh/shared';
import { useLocale } from '../../i18n';
import { useDebouncedValue } from '../../hooks/useDebouncedValue';
import { articleFilterParams, hasArticleFilters, parseLevel, type ParsedArticleQuery } from '../../lib/articleQuery';
import { btnSecondary, inputClass, selectClass } from '../../lib/ui';
import { useBrowse } from './BrowseContext';

function Field({ label, children }: { label: string; children: (id: string) => ReactNode }) {
  const id = useId();
  return (
    <div className="min-w-0">
      <label htmlFor={id} className="mb-2 block text-sm font-semibold text-ink">
        {label}
      </label>
      {children(id)}
    </div>
  );
}

/**
 * Thanh lọc danh sách bài — mọi bộ lọc nằm trên URL (chia sẻ/đánh dấu được, nút Back hoạt động), kết quả do máy chủ render.
 * Dùng cho /articles và /topics/[slug] (truyền `fixedCategory` thì ẩn ô chủ đề).
 * Giá trị trên ô đổi ngay (useOptimistic) trong lúc chờ máy chủ trả danh sách mới.
 */
export function ArticleFilters({
  basePath,
  query,
  fixedCategory,
  categories,
  tags,
}: {
  basePath: string;
  query: ParsedArticleQuery;
  fixedCategory?: string;
  categories: Category[] | null;
  tags: string[];
}) {
  const { t } = useTranslation();
  const locale = useLocale();
  const { navigate } = useBrowse();
  const [current, setCurrent] = useOptimistic(query);

  // Ô tìm kiếm: gõ ⇒ cập nhật URL sau 400 ms; URL đổi từ ngoài (ô tìm ở thanh đầu) ⇒ cập nhật ô.
  const urlQ = query.q ?? '';
  const [search, setSearch] = useState(urlQ);
  // So theo bản đã trim: đang gõ "hello " thì URL là "hello" — không được xoá khoảng trắng người dùng vừa gõ.
  useEffect(() => setSearch((s) => (s.trim() === urlQ.trim() ? s : urlQ)), [urlQ]);
  const debounced = useDebouncedValue(search, 400);

  const update = (changes: Partial<Record<'category' | 'level' | 'tag' | 'q' | 'sort', string | undefined>>) => {
    const next: ParsedArticleQuery = { ...current, ...changes, page: 1 } as ParsedArticleQuery;
    for (const [k, v] of Object.entries(changes)) if (!v) delete (next as unknown as Record<string, unknown>)[k];
    if (!next.sort) next.sort = 'latest';
    next.level = parseLevel(next.level);
    if (!next.level) delete next.level;
    const s = new URLSearchParams(articleFilterParams(next, fixedCategory)).toString();
    navigate(s ? `${basePath}?${s}` : basePath, () => setCurrent(next));
  };

  useEffect(() => {
    if (debounced.trim() !== urlQ.trim()) update({ q: debounced.trim() || undefined });
    // Chỉ phản ứng khi giá trị đã debounce đổi.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [debounced]);

  const submitSearch = (e: FormEvent) => {
    e.preventDefault();
    update({ q: search.trim() || undefined });
  };

  const filtered = hasArticleFilters(current, fixedCategory);
  const clearFilters = () => {
    setSearch('');
    navigate(basePath, () => setCurrent({ sort: 'latest', page: 1, ...(fixedCategory ? { category: fixedCategory } : {}) }));
  };

  const selectedCategory = categories?.find(c => c.slug === current.category);
  const activeFilters = [
    ...(current.q ? [{ key: 'q' as const, label: current.q }] : []),
    ...(!fixedCategory && current.category ? [{ key: 'category' as const, label: selectedCategory ? pickLocalized(selectedCategory.name, locale) : current.category }] : []),
    ...(current.level ? [{ key: 'level' as const, label: t(`common.levels.${current.level}`) }] : []),
    ...(current.tag ? [{ key: 'tag' as const, label: `#${current.tag}` }] : []),
    ...(current.sort !== 'latest' ? [{ key: 'sort' as const, label: t(current.sort === 'popular' ? 'articles.sortPopular' : 'articles.sortTrending') }] : []),
  ];
  const tagOptions = [...tags];
  if (current.tag && !tagOptions.includes(current.tag)) tagOptions.unshift(current.tag);

  return (
    <div className="space-y-5 rounded-card border border-line bg-surface p-4 sm:p-5">
      <form role="search" action={basePath} method="get" onSubmit={submitSearch} className="space-y-2">
        <label htmlFor="articles-search" className="block text-sm font-semibold text-ink">
          {t('articles.searchLabel')}
        </label>
        <div className="flex flex-col gap-3 sm:flex-row">
          <div className="relative min-w-0 flex-1">
          <Search aria-hidden="true" className="pointer-events-none absolute top-1/2 left-4 h-5 w-5 -translate-y-1/2 text-muted" />
          <input
            id="articles-search"
            name="q"
            type="search"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder={t('articles.searchPlaceholder')}
            enterKeyHint="search"
            className={`${inputClass} h-12 rounded-card pl-12 text-base`}
          />
          </div>
          <button type="submit" className={`${btnSecondary} min-h-12`}>{t('articles.searchSubmit')}</button>
        </div>
      </form>

      <div role="group" aria-label={t('articles.filtersLabel')} className={`grid gap-4 sm:grid-cols-2 ${fixedCategory ? 'lg:grid-cols-3' : 'lg:grid-cols-4'}`}>
        {!fixedCategory && (
          <Field label={t('articles.category')}>
            {(id) => (
              <select id={id} className={`${selectClass} min-h-11 text-base`} value={current.category ?? ''} onChange={(e) => update({ category: e.target.value || undefined })}>
                <option value="">{t('articles.allCategories')}</option>
                {categoryOptions(categories ?? [], locale).map(({ category: c, label }) => (
                  <option key={c.id} value={c.slug}>
                    {label}
                  </option>
                ))}
                {current.category && !categories?.some((c) => c.slug === current.category) && (
                  <option value={current.category}>{current.category}</option>
                )}
              </select>
            )}
          </Field>
        )}
        <Field label={t('articles.level')}>
          {(id) => (
            <select id={id} className={`${selectClass} min-h-11 text-base`} value={current.level ?? ''} onChange={(e) => update({ level: e.target.value || undefined })}>
              <option value="">{t('common.allLevels')}</option>
              {LEVELS.map((l) => (
                <option key={l} value={l}>
                  {t(`common.levels.${l}`)}
                </option>
              ))}
            </select>
          )}
        </Field>
        <Field label={t('articles.tag')}>
          {(id) => (
            <select id={id} className={`${selectClass} min-h-11 text-base`} value={current.tag ?? ''} onChange={(e) => update({ tag: e.target.value || undefined })}>
              <option value="">{t('articles.allTags')}</option>
              {tagOptions.map((tag) => (
                <option key={tag} value={tag}>
                  #{tag}
                </option>
              ))}
            </select>
          )}
        </Field>
        <Field label={t('articles.sort')}>
          {(id) => (
            <select id={id} className={`${selectClass} min-h-11 text-base`} value={current.sort} onChange={(e) => update({ sort: e.target.value === 'latest' ? undefined : e.target.value })}>
              <option value="latest">{t('articles.sortLatest')}</option>
              <option value="popular">{t('articles.sortPopular')}</option>
              <option value="trending">{t('articles.sortTrending')}</option>
            </select>
          )}
        </Field>
      </div>
      {filtered && <div className="flex flex-wrap items-center gap-2 border-t border-line pt-4">
        <span className="w-full text-sm font-semibold text-ink">{t('articles.activeFilters')}</span>
        {activeFilters.map(({ key, label }) => <button key={key} type="button" aria-label={t('articles.removeFilter', { label })} onClick={() => { if (key === 'q') setSearch(''); update({ [key]: undefined }); }} className="inline-flex min-h-11 max-w-full items-center gap-2 rounded-ui bg-primary-soft px-3 py-2 text-left text-sm text-primary-ink">
          <span className="[overflow-wrap:anywhere]">{label}</span><X aria-hidden="true" className="h-4 w-4 shrink-0" />
        </button>)}
          <button type="button" onClick={clearFilters} className={`${btnSecondary} min-h-11`}>
            <X aria-hidden="true" className="h-4 w-4" />
            {t('articles.clearFilters')}
          </button>
      </div>}
    </div>
  );
}
