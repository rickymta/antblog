import Link from 'next/link';
import { SearchX } from 'lucide-react';
import { formatNumber, type ArticleCard as ArticleCardData, type Page } from '@tkh/shared';
import { getServerI18n } from '../../i18n/server';
import { btnSecondary } from '../../lib/ui';
import { ArticleCard } from '../common/ArticleCard';
import { Pagination } from '../common/Pagination';
import { EmptyState, RefreshErrorState } from '../common/States';
import { BrowseResults } from './BrowseContext';

/** Kết quả danh sách bài (Server Component): số bài, lưới thẻ, phân trang bằng liên kết thật. `data = null` ⇒ lỗi tải. */
export async function ArticleResults({
  data,
  basePath,
  params,
  filtered,
  headingLevel = 2,
}: {
  data: Page<ArticleCardData> | null;
  basePath: string;
  /** Tham số lọc hiện tại (không có `page`). */
  params: Record<string, string>;
  filtered: boolean;
  headingLevel?: 2 | 3;
}) {
  const { t, locale } = await getServerI18n();
  if (!data) return <RefreshErrorState />;

  return (
    <BrowseResults className="space-y-6">
      <p className="text-sm text-muted" role="status">
        {t('articles.resultCount', { count: data.total, n: formatNumber(data.total, locale) })}
      </p>
      {data.items.length === 0 ? (
        <EmptyState
          icon={<SearchX aria-hidden="true" className="h-7 w-7 text-muted" />}
          title={t('articles.empty')}
          hint={filtered ? t('articles.emptyHint') : undefined}
          action={
            filtered ? (
              <Link href={basePath} scroll={false} className={btnSecondary}>
                {t('articles.clearFilters')}
              </Link>
            ) : undefined
          }
        />
      ) : (
        <>
          <ul className="grid gap-5 sm:grid-cols-2 lg:grid-cols-3">
            {data.items.map((a, i) => (
              <li key={a.id}>
                <ArticleCard article={a} showSummary headingLevel={headingLevel} eagerCover={i < 3} />
              </li>
            ))}
          </ul>
          <Pagination page={data.page} total={data.total} pageSize={data.pageSize} basePath={basePath} params={params} />
        </>
      )}
    </BrowseResults>
  );
}
