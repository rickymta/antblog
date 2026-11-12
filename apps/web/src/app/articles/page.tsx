import Link from 'next/link';
import { pickLocalized } from '@tkh/shared';
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
  const { t, locale } = await getServerI18n();
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
      {categories && <nav aria-label={t('articles.browseTopics')} className="mb-8">
        <h2 className="mb-3 text-lg font-semibold text-ink">{t('articles.browseTopics')}</h2>
        <ul className="flex flex-wrap gap-2">
          {categories.filter(c => !c.parentId && c.articleCount > 0).sort((a, b) => a.order - b.order).map(c => <li key={c.id}><Link href={`/topics/${encodeURIComponent(c.slug)}`} className="inline-flex min-h-11 items-center rounded-full border border-line px-4 py-2 text-sm text-primary-ink hover:bg-primary-soft">{pickLocalized(c.name, locale)}</Link></li>)}
        </ul>
      </nav>}
      <p className="mb-5 max-w-3xl text-base leading-relaxed text-muted">{t('articles.browseGuide')}</p>
      <BrowseProvider>
        <div className="space-y-6">
          <ArticleFilters basePath="/articles" query={query} categories={categories} tags={tags?.map((x) => x.tag) ?? []} />
          <ArticleResults data={articles} basePath="/articles" params={params} filtered={hasArticleFilters(query)} />
        </div>
      </BrowseProvider>
    </div>
  );
}
