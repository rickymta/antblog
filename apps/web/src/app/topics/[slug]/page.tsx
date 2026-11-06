import Link from 'next/link';
import type { Metadata } from 'next';
import { formatNumber, pickLocalized } from '@tkh/shared';
import { ArticleFilters } from '../../../components/browse/ArticleFilters';
import { ArticleResults } from '../../../components/browse/ArticleResults';
import { BrowseProvider } from '../../../components/browse/BrowseContext';
import { CategoryBanner } from '../../../components/topics/CategoryBanner';
import { ChildCategoryCard } from '../../../components/topics/ChildCategoryCard';
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
  const categories = await settle(serverApi.categories());
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
  // Nếu không tải được cây chủ đề (`categories = null`) thì coi như chủ đề lá: vẫn hiện danh sách bài (phương án an toàn).
  const [category, categories] = await Promise.all([orNotFound(serverApi.category(slug)), settle(serverApi.categories())]);
  const children = (categories ?? []).filter(c => c.parentId === category.id); // API đã sắp theo `order`, giữ nguyên thứ tự.
  const isParent = children.length > 0;
  const [articles, tags] = isParent
    ? [null, null]
    : await Promise.all([settle(serverApi.articles({ ...query, pageSize: ARTICLES_PAGE_SIZE })), settle(serverApi.popularTags(30))]);
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
      {!!ancestors.length && <nav aria-label={t('article.breadcrumb')} className="flex flex-wrap items-center gap-2 pt-6 text-sm text-muted"><Link href="/articles">{t('nav.articles')}</Link>{ancestors.map(c => <span key={c.id} className="inline-flex items-center gap-2"><span aria-hidden="true">/</span><Link href={topicPath(c.slug)} className="hover:text-primary-ink">{pickLocalized(c.name,locale)}</Link></span>)}<span aria-hidden="true">/</span><span aria-current="page" className="text-ink">{name}</span></nav>}
      <div className={`${ancestors.length ? 'pt-4' : 'pt-6'} pb-8`}>
        <CategoryBanner category={category} locale={locale} eyebrow={t('topic.eyebrow')} meta={meta} />
      </div>
      {isParent && (
        <section aria-labelledby="topic-children" className="pb-10">
          <h2 id="topic-children" className="sr-only">{t('topic.children')}</h2>
          <ul className="grid gap-5 sm:grid-cols-2 lg:grid-cols-3">
            {children.map(c => <li key={c.id}><ChildCategoryCard category={c} locale={locale} t={t} /></li>)}
          </ul>
        </section>
      )}
      {!isParent && <BrowseProvider>
        <div className="space-y-6">
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
