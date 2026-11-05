import Link from 'next/link';
import type { Metadata } from 'next';
import { CategoryGlyph } from '@tkh/ui';
import { formatNumber, pickLocalized } from '@tkh/shared';
import { ArticleFilters } from '../../../components/browse/ArticleFilters';
import { ArticleResults } from '../../../components/browse/ArticleResults';
import { BrowseProvider } from '../../../components/browse/BrowseContext';
import { getServerI18n } from '../../../i18n/server';
import {
  ARTICLES_PAGE_SIZE,
  articleFilterParams,
  hasArticleFilters,
  listCanonical,
  parseArticleQuery,
  type SearchParamsRecord,
} from '../../../lib/articleQuery';
import { safeColor, withAlpha } from '../../../lib/format';
import { breadcrumbList, JsonLd } from '../../../lib/jsonld';
import { clampDescription, notFoundMetadata, pageMetadata } from '../../../lib/metadata';
import { orNotFound, orNull, serverApi, settle } from '../../../lib/server-api';
import { absoluteUrl, SITE_URL } from '../../../lib/site';
import { container } from '../../../lib/ui';

type Props = { params: Promise<{ slug: string }>; searchParams: Promise<SearchParamsRecord> };

const topicPath = (slug: string) => `/topics/${encodeURIComponent(slug)}`;

export async function generateMetadata({ params, searchParams }: Props): Promise<Metadata> {
  const { slug } = await params;
  const { t, locale } = await getServerI18n();
  const category = await orNull(serverApi.category(slug));
  if (!category) return notFoundMetadata();
  const query = parseArticleQuery(await searchParams, category.slug);
  const name = pickLocalized(category.name, locale);
  return pageMetadata({
    title: query.page > 1 ? `${name} (${t('meta.pageN', { n: query.page })})` : name,
    description: clampDescription(pickLocalized(category.description, locale) || t('meta.description')),
    path: listCanonical(topicPath(category.slug), query.page, hasArticleFilters(query, category.slug)),
    locale,
    noindex: Boolean(query.q),
  });
}

export default async function TopicPage({ params, searchParams }: Props) {
  const { slug } = await params;
  const { t, locale } = await getServerI18n();
  const query = parseArticleQuery(await searchParams, slug);
  // Danh sách bài tải song song với thông tin chủ đề (lọc theo slug trên URL).
  const [category, articles, tags, categories] = await Promise.all([
    orNotFound(serverApi.category(slug)),
    settle(serverApi.articles({ ...query, pageSize: ARTICLES_PAGE_SIZE })),
    settle(serverApi.popularTags(30)),
    settle(serverApi.categories()),
  ]);
  const name = pickLocalized(category.name, locale);
  const color = safeColor(category.color);
  const basePath = topicPath(category.slug);
  const children = (categories ?? []).filter(c => c.parentId === category.id);
  const ancestors = [];
  let parent = (categories ?? []).find(c => c.id === category.parentId);
  const visited = new Set([category.id]);
  while (parent && !visited.has(parent.id)) {
    ancestors.unshift(parent);
    visited.add(parent.id);
    parent = (categories ?? []).find(c => c.id === parent?.parentId);
  }

  return (
    <div className={container}>
      <JsonLd
        data={breadcrumbList([
          { name: t('nav.home'), url: `${SITE_URL}/` },
          { name: t('nav.articles'), url: absoluteUrl('/articles') },
          ...ancestors.map(c => ({ name: pickLocalized(c.name, locale), url: absoluteUrl(topicPath(c.slug)) })),
          { name, url: absoluteUrl(basePath) },
        ])}
      />
      {!!ancestors.length && <nav aria-label={t('article.breadcrumb')} className="flex flex-wrap items-center gap-2 pt-6 text-sm text-muted"><Link href="/articles">{t('nav.articles')}</Link>{ancestors.map(c => <span key={c.id} className="inline-flex items-center gap-2"><span aria-hidden="true">/</span><Link href={topicPath(c.slug)} className="hover:text-primary-ink">{pickLocalized(c.name,locale)}</Link></span>)}<span aria-hidden="true">/</span><span aria-current="page" className="text-ink">{name}</span></nav>}
      <header className="flex flex-col gap-5 py-10 sm:flex-row sm:items-center">
        <span className="inline-flex h-16 w-16 shrink-0 items-center justify-center rounded-card" style={{ backgroundColor: withAlpha(color, 0.15) }}>
          <CategoryGlyph icon={category.icon} className="h-9 w-9" style={{ color }} />
        </span>
        <div className="min-w-0 space-y-2">
          <p className="text-xs font-bold tracking-wide text-muted uppercase">{t('topic.eyebrow')}</p>
          <h1 className="text-3xl font-extrabold tracking-tight text-ink sm:text-4xl">{name}</h1>
          <p className="max-w-3xl text-muted">{pickLocalized(category.description, locale)}</p>
          <p className="text-sm font-semibold text-ink">
            {t('common.articleCount', { count: category.articleCount, n: formatNumber(category.articleCount, locale) })}
          </p>
        </div>
      </header>
      {!!children.length && <nav aria-label={t('topic.children')} className="mb-7"><p className="mb-3 text-xs font-semibold text-muted">{t('topic.children')}</p><ul className="flex flex-wrap gap-3">{children.map(c => <li key={c.id}><Link href={topicPath(c.slug)} className="inline-flex items-center gap-3 rounded-ui border border-line bg-surface px-4 py-3 text-sm font-medium hover:border-primary"><CategoryGlyph icon={c.icon} className="h-4 w-4 text-primary-ink" />{pickLocalized(c.name,locale)}<span className="text-xs text-muted">{formatNumber(c.articleCount,locale)}</span></Link></li>)}</ul></nav>}
      <BrowseProvider>
        <div className="space-y-6">
          <ArticleFilters basePath={basePath} query={query} fixedCategory={category.slug} categories={null} tags={tags?.map((x) => x.tag) ?? []} />
          <ArticleResults
            data={articles}
            basePath={basePath}
            params={articleFilterParams(query, category.slug)}
            filtered={hasArticleFilters(query, category.slug)}
          />
        </div>
      </BrowseProvider>
    </div>
  );
}
