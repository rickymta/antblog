import Link from 'next/link';
import { ArrowRight, SearchX } from 'lucide-react';
import { formatNumber, type ArticleCard as ArticleCardData, type Page } from '@tkh/shared';
import { getServerI18n } from '../../i18n/server';
import { btnLink, btnSecondary, card } from '../../lib/ui';
import { Pagination } from '../common/Pagination';
import { pageHref } from '../../lib/navigation';
import { EmptyState, RefreshErrorState } from '../common/States';
import { ArticleListRow } from './ArticleListRow';
import { BrowseResults } from './BrowseContext';

// Viết lại tại đây: `pathHref` nằm trong mô-đun client (PathCard) nên Server Component không gọi được.
const pathHref = (slug: string) => `/learning-paths/${encodeURIComponent(slug)}`;

type Group = { key: string; ref?: ArticleCardData['pathRef']; items: ArticleCardData[] };

/** Gom các bài liền nhau cùng lộ trình thành một nhóm (danh sách đã xếp theo thứ tự học nên mỗi phần là một dải liền). */
function groupByPath(items: ArticleCardData[]): Group[] {
  const groups: Group[] = [];
  for (const a of items) {
    const key = a.pathRef?.slug ?? '';
    const last = groups[groups.length - 1];
    if (last && last.key === key) last.items.push(a);
    else groups.push({ key, ref: a.pathRef, items: [a] });
  }
  return groups;
}

/**
 * Kết quả danh sách bài (Server Component): số bài, danh sách dạng dòng, phân trang bằng liên kết thật. `data = null` ⇒ lỗi tải.
 * `byPath` (đang xếp theo lộ trình) ⇒ chia nhóm theo từng phần, mỗi nhóm có tiêu đề dẫn về lộ trình.
 */
export async function ArticleResults({
  data,
  basePath,
  params,
  filtered,
  byPath = false,
}: {
  data: Page<ArticleCardData> | null;
  basePath: string;
  /** Tham số lọc hiện tại (không có `page`). */
  params: Record<string, string>;
  filtered: boolean;
  byPath?: boolean;
}) {
  const { t, locale } = await getServerI18n();
  if (!data) return <RefreshErrorState />;
  const groups: Group[] = byPath ? groupByPath(data.items) : [{ key: 'all', items: data.items }];

  return (
    <BrowseResults className="space-y-5">
      <div className="flex flex-wrap items-baseline justify-between gap-x-6 gap-y-1">
        <h2 className="text-xl font-bold text-ink">{t('articles.resultsTitle')}</h2>
        <p className="text-sm text-muted" role="status">
          {data.items.length > 0 ? t('articles.resultRange', { from: formatNumber((data.page - 1) * data.pageSize + 1, locale), to: formatNumber((data.page - 1) * data.pageSize + data.items.length, locale), total: formatNumber(data.total, locale) }) : t('articles.resultCount', { count: data.total, n: formatNumber(data.total, locale) })}
        </p>
      </div>
      {data.items.length === 0 ? (
        <EmptyState
          icon={<SearchX aria-hidden="true" className="h-7 w-7 text-muted" />}
          title={t(data.page > 1 ? 'articles.pageUnavailable' : filtered ? 'articles.empty' : 'articles.noArticles')}
          hint={filtered ? t('articles.emptyHint') : undefined}
          action={
            data.page > 1 ? <Link href={pageHref(basePath, params, 1)} className={btnSecondary}>{t('articles.firstPage')}</Link> : filtered ? (
              <Link href={basePath} scroll={false} className={btnSecondary}>
                {t('articles.clearFilters')}
              </Link>
            ) : undefined
          }
        />
      ) : (
        <>
          <div className="space-y-6">
            {groups.map((group, g) => <section key={`${group.key}-${g}`} aria-label={byPath ? (group.ref?.title ?? t('articles.outsidePaths')) : undefined} className={`${card} overflow-hidden`}>
              {byPath && <header className="flex flex-wrap items-center justify-between gap-x-4 gap-y-1 border-b border-line bg-surface-2 px-4 py-3 sm:px-5">
                <h3 className="text-base font-bold text-ink">{group.ref?.title ?? t('articles.outsidePaths')}</h3>
                {group.ref && <Link href={pathHref(group.ref.slug)} className={`${btnLink} min-h-11`}>
                  {t('paths.view')}<span className="sr-only">: {group.ref.title}</span><ArrowRight aria-hidden="true" className="h-4 w-4 shrink-0" />
                </Link>}
              </header>}
              <ul className="divide-y divide-line">
                {group.items.map((a, i) => <li key={a.id}>
                  <ArticleListRow article={a} showPath={!byPath} headingLevel={byPath ? 4 : 3} eagerCover={g === 0 && i < 4} />
                </li>)}
              </ul>
            </section>)}
          </div>
          <Pagination page={data.page} total={data.total} pageSize={data.pageSize} basePath={basePath} params={params} />
        </>
      )}
    </BrowseResults>
  );
}
