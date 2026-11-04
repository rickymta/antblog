'use client';

import { useEffect, useId, useOptimistic, useState, type FormEvent, type ReactNode } from 'react';
import { useTranslation } from 'react-i18next';
import { Search, X } from 'lucide-react';
import { LEVELS, pickLocalized, type Category } from '@tkh/shared';
import { useLocale } from '../../i18n';
import { useDebouncedValue } from '../../hooks/useDebouncedValue';
import { articleFilterParams, hasArticleFilters, parseLevel, type ParsedArticleQuery } from '../../lib/articleQuery';
import { btnSecondary, inputClass, selectClass } from '../../lib/ui';
import { useBrowse } from './BrowseContext';

function Field({ label, children }: { label: string; children: (id: string) => ReactNode }) {
  const id = useId();
  return (
    <div className="min-w-0 flex-1 basis-36">
      <label htmlFor={id} className="mb-1 block text-xs font-semibold text-muted">
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

  const tagOptions = [...tags];
  if (current.tag && !tagOptions.includes(current.tag)) tagOptions.unshift(current.tag);

  return (
    <div className="space-y-6">
      <form role="search" action={basePath} method="get" onSubmit={submitSearch} className="relative">
        <label htmlFor="articles-search" className="sr-only">
          {t('articles.searchLabel')}
        </label>
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
      </form>

      <div role="group" aria-label={t('articles.filtersLabel')} className="flex flex-wrap items-end gap-3">
        {!fixedCategory && (
          <Field label={t('articles.category')}>
            {(id) => (
              <select id={id} className={selectClass} value={current.category ?? ''} onChange={(e) => update({ category: e.target.value || undefined })}>
                <option value="">{t('articles.allCategories')}</option>
                {categories?.map((c) => (
                  <option key={c.id} value={c.slug}>
                    {pickLocalized(c.name, locale)}
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
            <select id={id} className={selectClass} value={current.level ?? ''} onChange={(e) => update({ level: e.target.value || undefined })}>
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
            <select id={id} className={selectClass} value={current.tag ?? ''} onChange={(e) => update({ tag: e.target.value || undefined })}>
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
            <select id={id} className={selectClass} value={current.sort} onChange={(e) => update({ sort: e.target.value === 'latest' ? undefined : e.target.value })}>
              <option value="latest">{t('articles.sortLatest')}</option>
              <option value="popular">{t('articles.sortPopular')}</option>
              <option value="trending">{t('articles.sortTrending')}</option>
            </select>
          )}
        </Field>
        {filtered && (
          <button type="button" onClick={clearFilters} className={btnSecondary}>
            <X aria-hidden="true" className="h-4 w-4" />
            {t('articles.clearFilters')}
          </button>
        )}
      </div>
    </div>
  );
}
