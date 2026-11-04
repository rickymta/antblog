import type { Metadata } from 'next';
import { ArticleContent, extractHeadings } from '@tkh/ui';
import { pickLocalized, type ArticleDetail } from '@tkh/shared';
import { ArticleBody } from '../../../components/article/ArticleBody';
import { ArticleProvider } from '../../../components/article/ArticleContext';
import { ArticleHeader } from '../../../components/article/ArticleHeader';
import { PrevNext, RelatedArticles, ShareBlock, TagsBlock } from '../../../components/article/Aside';
import { PathNav } from '../../../components/article/PathNav';
import { ReactionCards } from '../../../components/article/ReactionCards';
import { TocCollapsible, TocPanel } from '../../../components/article/Toc';
import { CommentsSection } from '../../../components/comments/CommentsSection';
import { getServerI18n } from '../../../i18n/server';
import { breadcrumbList, JsonLd } from '../../../lib/jsonld';
import { clampDescription, notFoundMetadata, pageMetadata } from '../../../lib/metadata';
import { orNotFound, orNull, serverApi, settle } from '../../../lib/server-api';
import { absoluteUrl, LOGO_PATH, SITE_NAME, SITE_URL } from '../../../lib/site';
import { card, container } from '../../../lib/ui';

type Props = { params: Promise<{ slug: string }> };

const stickyAside = 'sticky top-24 max-h-[calc(100dvh-7rem)] space-y-8 overflow-y-auto overscroll-contain pr-1 pb-4';
// Trang bài rộng hơn khung chung (max-w-7xl) để cột giữa đạt 760px ở màn ≥ 1400px — hình SVG trong bài cần ~720px mới khỏi cuộn ngang.
const articleGrid = `${container} min-[1400px]:max-w-[1440px] grid gap-10 py-8 lg:grid-cols-[minmax(0,1fr)_16rem] xl:grid-cols-[15rem_minmax(0,1fr)_16rem]`;

const articlePath = (slug: string) => `/articles/${encodeURIComponent(slug)}`;

/** Ảnh chia sẻ: ảnh SEO riêng › ảnh bìa › ảnh sinh tự động từ tiêu đề bài. */
function shareImage(article: ArticleDetail): { url: string; generated: boolean } {
  const custom = article.seo?.ogImageUrl || article.coverImageUrl;
  if (custom) return { url: absoluteUrl(custom), generated: false };
  return { url: absoluteUrl(`/og/articles/${encodeURIComponent(article.slug)}`), generated: true };
}

export async function generateMetadata({ params }: Props): Promise<Metadata> {
  const { slug } = await params;
  const { locale } = await getServerI18n();
  const article = await orNull(serverApi.article(slug));
  if (!article) return notFoundMetadata();
  const description = clampDescription(article.seo?.description || article.summary || article.title);
  const image = shareImage(article);
  const section = pickLocalized(article.category.name, locale);
  return pageMetadata({
    ...(article.seo?.title ? { absoluteTitle: article.seo.title } : { title: article.title }),
    description,
    path: articlePath(article.slug),
    locale,
    image: { url: image.url, alt: article.title, ...(image.generated ? { width: 1200, height: 630 } : {}) },
    noindex: article.seo?.noindex === true,
    openGraph: {
      type: 'article',
      publishedTime: article.publishedAt,
      modifiedTime: article.updatedAt,
      section,
      tags: article.tags,
      authors: [article.author.displayName],
    },
  });
}

export default async function ArticlePage({ params }: Props) {
  const { slug } = await params;
  const { t, locale } = await getServerI18n();
  const [article, related] = await Promise.all([orNotFound(serverApi.article(slug)), settle(serverApi.related(slug, 3))]);
  const headings = extractHeadings(article.content);
  const path = article.learningPath;
  const url = absoluteUrl(articlePath(article.slug));
  const categoryName = pickLocalized(article.category.name, locale);
  const relatedArticles = related ?? [];

  const blogPosting = {
    '@context': 'https://schema.org',
    '@type': 'BlogPosting',
    headline: article.seo?.title || article.title,
    description: clampDescription(article.seo?.description || article.summary || article.title),
    datePublished: article.publishedAt,
    dateModified: article.updatedAt || article.publishedAt,
    author: { '@type': 'Person', name: article.author.displayName },
    publisher: {
      '@type': 'Organization',
      '@id': `${SITE_URL}/#organization`,
      name: SITE_NAME,
      url: `${SITE_URL}/`,
      logo: { '@type': 'ImageObject', url: absoluteUrl(LOGO_PATH), width: 512, height: 512 },
    },
    image: [shareImage(article).url],
    mainEntityOfPage: { '@type': 'WebPage', '@id': url },
    url,
    inLanguage: article.language,
    articleSection: categoryName,
    keywords: article.tags.join(', '),
    wordCount: article.content.split(/\s+/).filter(Boolean).length,
    timeRequired: `PT${Math.max(1, article.readingMinutes)}M`,
    ...(path ? { isPartOf: { '@type': 'CreativeWorkSeries', name: path.title, url: absoluteUrl(`/learning-paths/${encodeURIComponent(path.slug)}`) } } : {}),
  };
  const breadcrumbs = breadcrumbList([
    { name: t('nav.home'), url: `${SITE_URL}/` },
    { name: t('nav.articles'), url: absoluteUrl('/articles') },
    { name: categoryName, url: absoluteUrl(`/topics/${encodeURIComponent(article.category.slug)}`) },
    { name: article.title, url },
  ]);

  return (
    <>
      <JsonLd data={blogPosting} />
      <JsonLd data={breadcrumbs} />
      <ArticleProvider key={article.slug} initial={article}>
      <div className={articleGrid}>
        <aside className="hidden xl:block" aria-label={path ? t('article.pathNav') : t('article.toc')}>
          <div className={stickyAside}>{path ? <PathNav path={path} currentSlug={article.slug} /> : <TocPanel headings={headings} />}</div>
        </aside>

        <article className="mx-auto w-full max-w-[760px] min-w-0">
          <ArticleHeader />

          {(path || headings.length > 0) && (
            <div className="mt-6 space-y-3 lg:hidden">
              {path && (
                <details className={`group ${card} px-4 py-3`}>
                  <summary className="cursor-pointer text-sm font-semibold text-ink">
                    {t('article.inPath')}: {path.title}
                  </summary>
                  <div className="mt-3">
                    <PathNav path={path} currentSlug={article.slug} />
                  </div>
                </details>
              )}
              <TocCollapsible headings={headings} />
            </div>
          )}

          {article.coverImageUrl && (
            <div className="mt-8 aspect-[16/9] overflow-hidden rounded-card border border-line bg-surface-2">
              {/* eslint-disable-next-line @next/next/no-img-element -- ảnh bìa có thể là URL ngoài hoặc /uploads; khung tỉ lệ cố định chống nhảy trang */}
              <img
                src={article.coverImageUrl}
                alt={article.title}
                width={1280}
                height={720}
                fetchPriority="high"
                decoding="async"
                className="h-full w-full object-cover"
                referrerPolicy="no-referrer"
              />
            </div>
          )}

          <ArticleBody>
            <ArticleContent markdown={article.content} />
          </ArticleBody>

          <div className="mt-10 lg:hidden">
            <TagsBlock tags={article.tags} headingId="tags-title-inline" />
          </div>

          <div className="mt-12 space-y-10">
            <ReactionCards />
            <PrevNext article={article} />
            <div className="lg:hidden">
              <RelatedArticles articles={relatedArticles} headingId="related-title-inline" />
            </div>
            <CommentsSection slug={article.slug} />
          </div>
        </article>

        <aside className="hidden lg:block" aria-label={t('article.toc')}>
          <div className={stickyAside}>
            {headings.length > 0 && (
              <div className={path ? '' : 'xl:hidden'}>
                <TocPanel headings={headings} />
              </div>
            )}
            {path && (
              <div className="xl:hidden">
                <PathNav path={path} currentSlug={article.slug} />
              </div>
            )}
            <RelatedArticles articles={relatedArticles} />
            <TagsBlock tags={article.tags} />
            <ShareBlock />
          </div>
        </aside>
      </div>
      </ArticleProvider>
    </>
  );
}
