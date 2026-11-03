import { useEffect, useId, useState, type FormEvent, type ReactNode } from 'react';
import { useSearchParams } from 'react-router';
import { useTranslation } from 'react-i18next';
import { keepPreviousData, useQuery } from '@tanstack/react-query';
import { Search, SearchX, X } from 'lucide-react';
import { api, formatNumber, LEVELS, pickLocalized, type ArticleQuery, type ArticleSort, type Level } from '@tkh/shared';
import { useLocale } from '../../i18n';
import { useDebouncedValue } from '../../hooks/useDebouncedValue';
import { btnSecondary, inputClass, selectClass } from '../../lib/ui';
import { ArticleCard, ArticleCardSkeleton } from './ArticleCard';
import { Pagination } from './Pagination';
import { EmptyState, ErrorState, LoadingRegion } from './States';
import { categoriesQuery } from '../layout/TopicsMenu';

const PAGE_SIZE = 12;
const SORTS: readonly ArticleSort[] = ['latest', 'popular', 'trending'];

function parseQuery(params: URLSearchParams, fixedCategory?: string): ArticleQuery {
  const level = params.get('level');
  const sort = params.get('sort');
  const page = Number.parseInt(params.get('page') ?? '1', 10);
  return {
    category: fixedCategory ?? (params.get('category') || undefined),
    level: LEVELS.includes(level as Level) ? (level as Level) : undefined,
    tag: params.get('tag') || undefined,
    q: params.get('q')?.trim() || undefined,
    sort: SORTS.includes(sort as ArticleSort) ? (sort as ArticleSort) : 'latest',
    page: Number.isFinite(page) && page > 0 ? page : 1,
    pageSize: PAGE_SIZE,
  };
}

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
 * Danh sách bài có lọc — mọi bộ lọc nằm trên URL (chia sẻ/đánh dấu được, nút Back hoạt động).
 * Dùng cho /articles và /topics/:slug (truyền `fixedCategory` thì ẩn ô chủ đề).
 */
export function ArticleBrowser({ fixedCategory, headingLevel = 2 }: { fixedCategory?: string; headingLevel?: 2 | 3 }) {
  const { t } = useTranslation();
  const locale = useLocale();
  const [params, setParams] = useSearchParams();
  const query = parseQuery(params, fixedCategory);

  const categories = useQuery({ ...categoriesQuery, enabled: !fixedCategory });
  const tags = useQuery({ queryKey: ['tags', 'popular', 30], queryFn: () => api.tags.popular(30), staleTime: 5 * 60_000 });
  const articles = useQuery({
    queryKey: ['articles', 'list', query],
    queryFn: () => api.articles.list(query),
    placeholderData: keepPreviousData,
  });

  // Ô tìm kiếm: gõ ⇒ cập nhật URL sau 400 ms; URL đổi từ ngoài (ô tìm ở thanh đầu) ⇒ cập nhật ô.
  const urlQ = params.get('q') ?? '';
  const [search, setSearch] = useState(urlQ);
  // So theo bản đã trim: đang gõ "hello " thì URL là "hello" — không được xoá khoảng trắng người dùng vừa gõ.
  useEffect(() => setSearch((s) => (s.trim() === urlQ.trim() ? s : urlQ)), [urlQ]);
  const debounced = useDebouncedValue(search, 400);

  const update = (changes: Record<string, string | undefined>, resetPage = true) => {
    setParams(
      (prev) => {
        const next = new URLSearchParams(prev);
        for (const [k, v] of Object.entries(changes)) {
          if (v) next.set(k, v);
          else next.delete(k);
        }
        if (resetPage) next.delete('page');
        return next;
      },
      { replace: true, preventScrollReset: true },
    );
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

  const hasFilters = Boolean((!fixedCategory && query.category) || query.level || query.tag || query.q || query.sort !== 'latest');
  const clearFilters = () => {
    setSearch('');
    setParams(new URLSearchParams(), { replace: true, preventScrollReset: true });
  };

  const hrefFor = (page: number) => {
    const next = new URLSearchParams(params);
    if (page > 1) next.set('page', String(page));
    else next.delete('page');
    const s = next.toString();
    return s ? `?${s}` : '?';
  };

  const tagOptions = tags.data?.map((x) => x.tag) ?? [];
  if (query.tag && !tagOptions.includes(query.tag)) tagOptions.unshift(query.tag);

  return (
    <div className="space-y-6">
      <form role="search" onSubmit={submitSearch} className="relative">
        <label htmlFor="articles-search" className="sr-only">
          {t('articles.searchLabel')}
        </label>
        <Search aria-hidden="true" className="pointer-events-none absolute top-1/2 left-4 h-5 w-5 -translate-y-1/2 text-muted" />
        <input
          id="articles-search"
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
              <select id={id} className={selectClass} value={query.category ?? ''} onChange={(e) => update({ category: e.target.value || undefined })}>
                <option value="">{t('articles.allCategories')}</option>
                {categories.data?.map((c) => (
                  <option key={c.id} value={c.slug}>
                    {pickLocalized(c.name, locale)}
                  </option>
                ))}
                {query.category && !categories.data?.some((c) => c.slug === query.category) && <option value={query.category}>{query.category}</option>}
              </select>
            )}
          </Field>
        )}
        <Field label={t('articles.level')}>
          {(id) => (
            <select id={id} className={selectClass} value={query.level ?? ''} onChange={(e) => update({ level: e.target.value || undefined })}>
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
            <select id={id} className={selectClass} value={query.tag ?? ''} onChange={(e) => update({ tag: e.target.value || undefined })}>
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
            <select
              id={id}
              className={selectClass}
              value={query.sort}
              onChange={(e) => update({ sort: e.target.value === 'latest' ? undefined : e.target.value })}
            >
              <option value="latest">{t('articles.sortLatest')}</option>
              <option value="popular">{t('articles.sortPopular')}</option>
              <option value="trending">{t('articles.sortTrending')}</option>
            </select>
          )}
        </Field>
        {hasFilters && (
          <button type="button" onClick={clearFilters} className={btnSecondary}>
            <X aria-hidden="true" className="h-4 w-4" />
            {t('articles.clearFilters')}
          </button>
        )}
      </div>

      {articles.data && (
        <p className="text-sm text-muted" role="status">
          {t('articles.resultCount', { count: articles.data.total, n: formatNumber(articles.data.total, locale) })}
        </p>
      )}

      {articles.isPending ? (
        <LoadingRegion className="grid gap-5 sm:grid-cols-2 lg:grid-cols-3">
          {Array.from({ length: 6 }, (_, i) => (
            <ArticleCardSkeleton key={i} />
          ))}
        </LoadingRegion>
      ) : articles.isError ? (
        <ErrorState onRetry={() => void articles.refetch()} />
      ) : articles.data.items.length === 0 ? (
        <EmptyState
          icon={<SearchX aria-hidden="true" className="h-7 w-7 text-muted" />}
          title={t('articles.empty')}
          hint={hasFilters ? t('articles.emptyHint') : undefined}
          action={
            hasFilters ? (
              <button type="button" onClick={clearFilters} className={btnSecondary}>
                {t('articles.clearFilters')}
              </button>
            ) : undefined
          }
        />
      ) : (
        <>
          <ul className={`grid gap-5 sm:grid-cols-2 lg:grid-cols-3 ${articles.isPlaceholderData ? 'opacity-60 transition-opacity' : ''}`}>
            {articles.data.items.map((a) => (
              <li key={a.id}>
                <ArticleCard article={a} showSummary headingLevel={headingLevel} />
              </li>
            ))}
          </ul>
          <Pagination page={articles.data.page} total={articles.data.total} pageSize={articles.data.pageSize} hrefFor={hrefFor} />
        </>
      )}
    </div>
  );
}
