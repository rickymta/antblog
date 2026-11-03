import { Link } from 'react-router';
import { keepPreviousData, useQuery } from '@tanstack/react-query';
import { useTranslation } from 'react-i18next';
import { Plus, Search } from 'lucide-react';
import { api, formatDate, formatNumber, pickLocalized, type ArticleStatus } from '@tkh/shared';
import {
  Button,
  Card,
  EmptyState,
  ErrorState,
  FilterBar,
  PageHeader,
  Pagination,
  TableSkeleton,
  TableWrap,
  Td,
  Th,
  buttonClass,
  inputClass,
  selectClass,
} from '../components/ui';
import { ArticleStatusBadge, CategoryChip, LevelBadge } from '../components/domain';
import { useSearchBox, useUrlFilters } from '../lib/hooks';
import { errorMessage } from '../lib/errors';
import { useCategoryOptions } from '../lib/queries';
import { useIsAdmin } from '../lib/session';
import { useLocale } from '../i18n';

const PAGE_SIZE = 20;
const STATUSES: ArticleStatus[] = ['draft', 'published', 'archived'];

export function ArticlesPage() {
  const { t, i18n } = useTranslation();
  const locale = useLocale();
  const isAdmin = useIsAdmin();
  const f = useUrlFilters(['status', 'category', 'q'] as const);
  const [text, setText] = useSearchBox(f.values.q, (v) => f.set('q', v));
  const categories = useCategoryOptions();

  const query = { status: f.values.status, category: f.values.category, q: f.values.q, page: f.page, pageSize: PAGE_SIZE };
  const list = useQuery({
    queryKey: ['admin', 'articles', 'list', query],
    queryFn: () => api.admin.articles.list(query),
    placeholderData: keepPreviousData,
  });

  return (
    <>
      <PageHeader
        title={t('articles.title')}
        subtitle={isAdmin ? t('articles.subtitle') : t('articles.authorSubtitle')}
        actions={
          <Link to="/articles/new" className={buttonClass('primary')}>
            <Plus className="size-4" aria-hidden />
            {t('articles.newArticle')}
          </Link>
        }
      />
      <Card>
        <FilterBar>
          <label className="relative min-w-[14rem] flex-[2]">
            <span className="sr-only">{t('common.search')}</span>
            <Search className="pointer-events-none absolute top-1/2 left-3 size-4 -translate-y-1/2 text-muted" aria-hidden />
            <input
              type="search"
              className={`${inputClass} h-10 pl-9`}
              placeholder={t('articles.searchPlaceholder')}
              value={text}
              onChange={(e) => setText(e.target.value)}
            />
          </label>
          <label className="min-w-[10rem] flex-1">
            <span className="mb-1 block text-xs font-medium text-muted">{t('articles.filterStatus')}</span>
            <select className={selectClass} value={f.values.status} onChange={(e) => f.set('status', e.target.value)}>
              {/* API: không lọc trạng thái ⇒ ẩn bài lưu trữ. */}
              <option value="">{t('articles.statusAllActive')}</option>
              {STATUSES.map((s) => (
                <option key={s} value={s}>
                  {t(`articleStatus.${s}`)}
                </option>
              ))}
            </select>
          </label>
          <label className="min-w-[10rem] flex-1">
            <span className="mb-1 block text-xs font-medium text-muted">{t('articles.filterCategory')}</span>
            <select className={selectClass} value={f.values.category} onChange={(e) => f.set('category', e.target.value)}>
              <option value="">{t('common.all')}</option>
              {categories.data?.map((c) => (
                <option key={c.id} value={c.slug}>
                  {pickLocalized(c.name, locale)}
                </option>
              ))}
            </select>
          </label>
          {f.active && (
            <Button variant="ghost" onClick={f.clear}>
              {t('common.clearFilters')}
            </Button>
          )}
        </FilterBar>

        {list.isPending ? (
          <TableSkeleton cols={6} />
        ) : list.isError ? (
          <ErrorState message={errorMessage(list.error, t, i18n.language)} onRetry={() => void list.refetch()} />
        ) : list.data.items.length === 0 ? (
          f.active ? (
            <EmptyState title={t('states.emptyFiltered')} action={<Button onClick={f.clear}>{t('common.clearFilters')}</Button>} />
          ) : (
            <EmptyState
              title={t('articles.empty')}
              body={t('articles.emptyHint')}
              action={
                <Link to="/articles/new" className={buttonClass('primary')}>
                  {t('articles.newArticle')}
                </Link>
              }
            />
          )
        ) : (
          <>
            <TableWrap label={t('articles.title')}>
              <thead>
                <tr>
                  <Th>{t('articles.colTitle')}</Th>
                  <Th>{t('articles.colCategory')}</Th>
                  <Th>{t('articles.colLevel')}</Th>
                  <Th>{t('articles.colStatus')}</Th>
                  <Th className="text-right">{t('articles.colViews')}</Th>
                  <Th className="text-right">{t('articles.colComments')}</Th>
                  <Th>{t('articles.colUpdated')}</Th>
                </tr>
              </thead>
              <tbody className={list.isPlaceholderData ? 'opacity-60' : undefined}>
                {list.data.items.map((a) => (
                  <tr key={a.id} className="hover:bg-surface-2/60">
                    <Td className="min-w-[16rem] max-w-[26rem]">
                      <Link to={`/articles/${a.id}`} className="line-clamp-2 font-medium text-primary-ink hover:underline">
                        {a.title}
                      </Link>
                      <span className="mt-0.5 block truncate font-mono text-xs text-muted">/{a.slug}</span>
                    </Td>
                    <Td>{a.category ? <CategoryChip name={pickLocalized(a.category.name, locale)} color={a.category.color} /> : t('common.none')}</Td>
                    <Td>
                      <LevelBadge level={a.level} />
                    </Td>
                    <Td>
                      <ArticleStatusBadge status={a.status} />
                    </Td>
                    <Td className="text-right tabular-nums">{formatNumber(a.stats?.views ?? 0, locale)}</Td>
                    <Td className="text-right tabular-nums">{formatNumber(a.stats?.comments ?? 0, locale)}</Td>
                    <Td className="whitespace-nowrap text-muted">{formatDate(a.updatedAt, locale)}</Td>
                  </tr>
                ))}
              </tbody>
            </TableWrap>
            <Pagination page={list.data.page} pageSize={list.data.pageSize} total={list.data.total} onPage={f.setPage} />
          </>
        )}
      </Card>
    </>
  );
}
