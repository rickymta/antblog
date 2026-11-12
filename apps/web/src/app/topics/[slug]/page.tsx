import Link from 'next/link';
import type { Metadata } from 'next';
import { formatNumber, pickLocalized } from '@tkh/shared';
import { ArticleFilters } from '../../../components/browse/ArticleFilters';
import { ArticleResults } from '../../../components/browse/ArticleResults';
import { BrowseProvider } from '../../../components/browse/BrowseContext';
import { CategoryBanner } from '../../../components/topics/CategoryBanner';
import { ChildCategoryCard } from '../../../components/topics/ChildCategoryCard';
import { TopicParts } from '../../../components/topics/TopicParts';
import { getServerI18n } from '../../../i18n/server';
import {
  ARTICLES_PAGE_SIZE,
  articleFilterParams,
  hasArticleFilters,
  listCanonical,
  parseArticleQuery,
  type SearchParamsRecord,
} from '../../../lib/articleQuery';
import { breadcrumbList, JsonLd } from '../../../lib/jsonld';
import { orderTopicParts } from '../../../lib/learning-paths';
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
  const name = pickLocalized(category.name, locale);
  const description = clampDescription(pickLocalized(category.description, locale) || t('meta.description'));
  // Chủ đề cha chỉ liệt kê chủ đề con (không có danh sách bài) ⇒ bỏ qua ?page=, ?q=, bộ lọc: một URL chuẩn duy nhất, luôn được index.
  // Cùng URL `/categories` với trang (fetch được Next dedupe trong một request) nên không tốn thêm lượt gọi API.
  const categories = await serverApi.categories();
  if (categories?.some(c => c.parentId === category.id)) {
    return pageMetadata({ title: name, description, path: topicPath(category.slug), locale });
  }
  const query = parseArticleQuery(await searchParams, category.slug);
  return pageMetadata({
    title: query.page > 1 ? `${name} (${t('meta.pageN', { n: query.page })})` : name,
    description,
    path: listCanonical(topicPath(category.slug), query.page, hasArticleFilters(query, category.slug)),
    locale,
    noindex: Boolean(query.q),
  });
}

export default async function TopicPage({ params, searchParams }: Props) {
  const { slug } = await params;
  const { t, locale } = await getServerI18n();
  const query = parseArticleQuery(await searchParams, slug);
  // Chủ đề + cây chủ đề tải song song. Chủ đề cha (có chủ đề con) chỉ liệt kê chủ đề con, KHÔNG gọi API bài viết/thẻ.
  // Cây chủ đề phải tải thành công trước khi quyết định có hiển thị bài viết.
  const [category, categories] = await Promise.all([orNotFound(serverApi.category(slug)), serverApi.categories()]);
  const children = (categories ?? []).filter(c => c.parentId === category.id); // API đã sắp theo `order`, giữ nguyên thứ tự.
  const availableChildren = children.filter(c => c.articleCount > 0);
  const upcomingChildren = children.filter(c => c.articleCount === 0);
  const isParent = children.length > 0;
  const [articles, tags] = isParent
    ? [null, null]
    : await Promise.all([settle(serverApi.articles({ ...query, pageSize: ARTICLES_PAGE_SIZE })), settle(serverApi.popularTags(30))]);
  // Các phần (lộ trình) của chính chủ đề này, theo thứ tự học. Ẩn khi đang tìm/lọc/sang trang để kết quả nằm ngay đầu trang.
  const browsing = !isParent && (hasArticleFilters(query, category.slug) || query.page > 1);
  const partCards = browsing ? [] : orderTopicParts(((await settle(serverApi.learningPaths())) ?? []).filter(p => p.category?.slug === category.slug && p.articleCount > 0));
  const parts = (await Promise.all(partCards.map(p => settle(serverApi.learningPath(p.slug))))).filter(p => p !== null && p.articles.length > 0).map(p => p!);
  const name = pickLocalized(category.name, locale);
  const basePath = topicPath(category.slug);
  const articleCountLabel = t('common.articleCount', { count: category.articleCount, n: formatNumber(category.articleCount, locale) });
  const meta = isParent ? `${articleCountLabel} · ${t('topic.childCount', { count: children.length, n: formatNumber(children.length, locale) })}` : articleCountLabel;
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
      <nav aria-label={t('article.breadcrumb')} className="pt-4 text-sm text-muted">
        <ol className="flex flex-wrap items-center gap-x-2 gap-y-1">
          <li><Link href="/" className="inline-flex min-h-11 items-center hover:text-primary-ink hover:underline">{t('nav.home')}</Link></li>
          <li className="flex items-center gap-2"><span aria-hidden="true">/</span><Link href="/articles" className="inline-flex min-h-11 items-center hover:text-primary-ink hover:underline">{t('nav.articles')}</Link></li>
          {ancestors.map(c => <li key={c.id} className="flex items-center gap-2"><span aria-hidden="true">/</span><Link href={topicPath(c.slug)} className="inline-flex min-h-11 items-center hover:text-primary-ink hover:underline">{pickLocalized(c.name, locale)}</Link></li>)}
          <li className="flex items-center gap-2"><span aria-hidden="true">/</span><span aria-current="page" className="text-ink">{name}</span></li>
        </ol>
      </nav>
      <div className="pt-4 pb-8">
        <CategoryBanner category={category} locale={locale} eyebrow={t('topic.eyebrow')} meta={meta} />
      </div>
      {parts.length > 0 && <TopicParts parts={parts} />}
      {isParent && (
        <div className="space-y-10 pb-10">
          <p className="max-w-3xl text-base leading-relaxed text-muted">{t('topic.chooseGuide')}</p>
          {availableChildren.length > 0 && <section aria-labelledby="topic-children">
            <h2 id="topic-children" className="mb-5 text-xl font-bold text-ink">{t('topic.children')}</h2>
            <ul className="grid gap-5 sm:grid-cols-2 lg:grid-cols-3">
              {availableChildren.map(c => <li key={c.id}><ChildCategoryCard category={c} locale={locale} t={t} /></li>)}
            </ul>
          </section>}
          {upcomingChildren.length > 0 && <section aria-labelledby="topic-upcoming">
            <h2 id="topic-upcoming" className="mb-5 text-xl font-bold text-ink">{t('topic.upcoming')}</h2>
            <ul className="grid gap-5 sm:grid-cols-2 lg:grid-cols-3">
              {upcomingChildren.map(c => <li key={c.id}><ChildCategoryCard category={c} locale={locale} t={t} /></li>)}
            </ul>
          </section>}
        </div>
      )}
      {!isParent && <BrowseProvider>
        <div className="space-y-6">
          <div className="max-w-3xl space-y-2">
            <p className="text-base leading-relaxed text-muted">{t(parts.length > 0 ? 'topic.searchGuide' : 'topic.articlesGuide')}</p>
            <Link href="/learning-paths" className="inline-flex min-h-11 items-center gap-2 text-sm font-semibold text-primary-ink hover:underline">{t('topic.viewPaths')} <span aria-hidden="true">→</span></Link>
          </div>
          <ArticleFilters basePath={basePath} query={query} fixedCategory={category.slug} categories={null} tags={tags?.map((x) => x.tag) ?? []} />
          <ArticleResults
            data={articles}
            basePath={basePath}
            params={articleFilterParams(query, category.slug)}
            filtered={hasArticleFilters(query, category.slug)}
          />
        </div>
      </BrowseProvider>}
    </div>
  );
}
