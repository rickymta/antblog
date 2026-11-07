import type { Metadata } from 'next';
import { ArticleFilters } from '../../components/browse/ArticleFilters';
import { ArticleResults } from '../../components/browse/ArticleResults';
import { BrowseProvider } from '../../components/browse/BrowseContext';
import { PageBanner } from '../../components/common/PageBanner';
import { getServerI18n } from '../../i18n/server';
import {
  ARTICLES_PAGE_SIZE,
  articleFilterParams,
  hasArticleFilters,
  listCanonical,
  parseArticleQuery,
  type SearchParamsRecord,
} from '../../lib/articleQuery';
import { pageMetadata } from '../../lib/metadata';
import { serverApi, settle } from '../../lib/server-api';
import { container } from '../../lib/ui';

type Props = { searchParams: Promise<SearchParamsRecord> };

export async function generateMetadata({ searchParams }: Props): Promise<Metadata> {
  const { t, locale } = await getServerI18n();
  const query = parseArticleQuery(await searchParams);
  const filtered = hasArticleFilters(query);
  const base = query.q ? t('meta.searchTitle', { q: query.q }) : t('articles.title');
  const title = query.page > 1 ? `${base} (${t('meta.pageN', { n: query.page })})` : base;
  return pageMetadata({
    title,
    description: t('articles.subtitle'),
    path: listCanonical('/articles', query.page, filtered),
    locale,
    // Trang kết quả tìm kiếm: không lập chỉ mục (nội dung mỏng, vô hạn tổ hợp) nhưng vẫn cho đi theo liên kết.
    noindex: Boolean(query.q),
  });
}

export default async function ArticlesPage({ searchParams }: Props) {
  const { t } = await getServerI18n();
  const query = parseArticleQuery(await searchParams);
  const [articles, categories, tags] = await Promise.all([
    settle(serverApi.articles({ ...query, pageSize: ARTICLES_PAGE_SIZE })),
    settle(serverApi.categories()),
    settle(serverApi.popularTags(30)),
  ]);
  const params = articleFilterParams(query);

  return (
    <div className={container}>
      <PageBanner variant="articles" title={t('articles.title')} subtitle={t('articles.subtitle')} />
      <BrowseProvider>
        <div className="space-y-6">
          <ArticleFilters basePath="/articles" query={query} categories={categories} tags={tags?.map((x) => x.tag) ?? []} />
          <ArticleResults data={articles} basePath="/articles" params={params} filtered={hasArticleFilters(query)} />
        </div>
      </BrowseProvider>
    </div>
  );
}
