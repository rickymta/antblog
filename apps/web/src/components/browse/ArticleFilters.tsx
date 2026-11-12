'use client';

import { useEffect, useId, useOptimistic, useState, type FormEvent, type ReactNode } from 'react';
import { useTranslation } from 'react-i18next';
import { Search, X } from 'lucide-react';
import { LEVELS, categoryOptions, pickLocalized, type Category } from '@tkh/shared';
import { useLocale } from '../../i18n';
import { useDebouncedValue } from '../../hooks/useDebouncedValue';
import { ARTICLE_SORTS, DEFAULT_ARTICLE_SORT, articleFilterParams, hasArticleFilters, parseLevel, type ParsedArticleQuery } from '../../lib/articleQuery';
import { btnGhost, inputClass, selectClass } from '../../lib/ui';
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
 * Dùng cho /articles và /topics/[slug] (truyền `fixedCategory` thì ẩn ô chủ đề). Một hàng: tìm kiếm · chủ đề · sắp xếp;
 * cấp độ là dãy nút bấm nhanh. Thẻ (`tag`) chỉ đến từ liên kết trong bài — hiện thành điều kiện đang lọc, không có ô chọn.
 * Giá trị trên ô đổi ngay (useOptimistic) trong lúc chờ máy chủ trả danh sách mới.
 */
export function ArticleFilters({
  basePath,
  query,
  fixedCategory,
  categories,
}: {
  basePath: string;
  query: ParsedArticleQuery;
  fixedCategory?: string;
  categories: Category[] | null;
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
    if (!next.sort) next.sort = DEFAULT_ARTICLE_SORT;
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
    navigate(basePath, () => setCurrent({ sort: DEFAULT_ARTICLE_SORT, page: 1, ...(fixedCategory ? { category: fixedCategory } : {}) }));
  };

  const selectedCategory = categories?.find(c => c.slug === current.category);
  const sortLabel = (sort: (typeof ARTICLE_SORTS)[number]) =>
    t(sort === 'path' ? 'articles.sortPath' : sort === 'popular' ? 'articles.sortPopular' : sort === 'trending' ? 'articles.sortTrending' : 'articles.sortLatest');
  // Cấp độ đã hiện trạng thái ngay trên dãy nút nên không lặp lại ở đây.
  const activeFilters = [
    ...(current.q ? [{ key: 'q' as const, label: current.q }] : []),
    ...(!fixedCategory && current.category ? [{ key: 'category' as const, label: selectedCategory ? pickLocalized(selectedCategory.name, locale) : current.category }] : []),
    ...(current.tag ? [{ key: 'tag' as const, label: `#${current.tag}` }] : []),
    ...(current.sort !== DEFAULT_ARTICLE_SORT ? [{ key: 'sort' as const, label: sortLabel(current.sort) }] : []),
  ];
  const levelChip = (active: boolean) =>
    `inline-flex min-h-11 items-center rounded-full border px-4 text-sm font-medium transition-colors ${active ? 'border-primary bg-primary text-white' : 'border-line bg-surface text-ink hover:bg-surface-2'}`;

  return (
    <div className="space-y-4 rounded-card border border-line bg-surface p-4 sm:p-5">
      <div className={`grid gap-4 ${fixedCategory ? 'lg:grid-cols-[minmax(0,1fr)_14rem]' : 'grid-cols-2 lg:grid-cols-[minmax(0,1fr)_15rem_13rem]'}`}>
        <form role="search" action={basePath} method="get" onSubmit={submitSearch} className={`min-w-0 ${fixedCategory ? '' : 'col-span-2 lg:col-span-1'}`}>
          <label htmlFor="articles-search" className="mb-2 block text-sm font-semibold text-ink">
            {t('articles.searchLabel')}
          </label>
          <div className="relative">
            <Search aria-hidden="true" className="pointer-events-none absolute top-1/2 left-3.5 h-5 w-5 -translate-y-1/2 text-muted" />
            <input
              id="articles-search"
              name="q"
              type="search"
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              placeholder={t('articles.searchPlaceholder')}
              enterKeyHint="search"
              className={`${inputClass} min-h-11 pl-11 text-base`}
            />
          </div>
          <button type="submit" className="sr-only">{t('articles.searchSubmit')}</button>
        </form>
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
        <Field label={t('articles.sort')}>
          {(id) => (
            <select id={id} className={`${selectClass} min-h-11 text-base`} value={current.sort} onChange={(e) => update({ sort: e.target.value === DEFAULT_ARTICLE_SORT ? undefined : e.target.value })}>
              {ARTICLE_SORTS.map((sort) => <option key={sort} value={sort}>{sortLabel(sort)}</option>)}
            </select>
          )}
        </Field>
      </div>

      <div role="group" aria-label={t('articles.level')} className="flex flex-wrap items-center gap-2">
        <button type="button" aria-pressed={!current.level} onClick={() => update({ level: undefined })} className={levelChip(!current.level)}>{t('common.allLevels')}</button>
        {LEVELS.map((l) => (
          <button key={l} type="button" aria-pressed={current.level === l} onClick={() => update({ level: current.level === l ? undefined : l })} className={levelChip(current.level === l)}>
            {t(`common.levels.${l}`)}
          </button>
        ))}
      </div>

      {filtered && <div className="flex flex-wrap items-center gap-2 border-t border-line pt-4">
        {activeFilters.length > 0 && <span className="text-sm font-semibold text-ink">{t('articles.activeFilters')}</span>}
        {activeFilters.map(({ key, label }) => <button key={key} type="button" aria-label={t('articles.removeFilter', { label })} onClick={() => { if (key === 'q') setSearch(''); update({ [key]: undefined }); }} className="inline-flex min-h-11 max-w-full items-center gap-2 rounded-ui bg-primary-soft px-3 py-2 text-left text-sm text-primary-ink">
          <span className="[overflow-wrap:anywhere]">{label}</span><X aria-hidden="true" className="h-4 w-4 shrink-0" />
        </button>)}
        <button type="button" onClick={clearFilters} className={`${btnGhost} min-h-11`}>
          <X aria-hidden="true" className="h-4 w-4" />
          {t('articles.clearFilters')}
        </button>
      </div>}
    </div>
  );
}
