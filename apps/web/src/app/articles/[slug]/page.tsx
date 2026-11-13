import type { Metadata } from 'next';
// Phông mã (khối code, nhãn ngôn ngữ, hình SVG trong bài) — chỉ trang bài cần ⇒ chỉ nạp ở đây, không nạp toàn site.
import '@fontsource/jetbrains-mono/400.css';
import { ArticleContent, extractHeadings } from '@tkh/ui';
import { pickLocalized, type ArticleDetail } from '@tkh/shared';
import { ArticleBody } from '../../../components/article/ArticleBody';
import { ArticleProvider } from '../../../components/article/ArticleContext';
import { ArticleHeader } from '../../../components/article/ArticleHeader';
import { PrevNext, RelatedArticles, TagsBlock } from '../../../components/article/Aside';
import { PathNav } from '../../../components/article/PathNav';
import { ReactionCards } from '../../../components/article/ReactionCards';
import { TocCollapsible, TocPanel } from '../../../components/article/Toc';
import { LazyComments } from '../../../components/comments/LazyComments';
import { I18N_ENABLED } from '../../../i18n/config';
import { getServerI18n } from '../../../i18n/server';
import { breadcrumbList, JsonLd } from '../../../lib/jsonld';
import { clampDescription, notFoundMetadata, pageMetadata } from '../../../lib/metadata';
import { orNotFound, orNull, serverApi, settle } from '../../../lib/server-api';
import { absoluteUrl, LOGO_PATH, SITE_NAME, SITE_URL } from '../../../lib/site';
import { card } from '../../../lib/ui';
import { articleCover } from '../../../lib/article-cover';

type Props = { params: Promise<{ slug: string }> };

/**
 * ISR: dựng trang bài lần đầu khi có request rồi cache ở máy chủ `revalidate` giây (HTML không phụ thuộc cookie/request).
 * generateStaticParams trả [] ⇒ KHÔNG dựng gì lúc `next build` (ảnh Docker build không có API); slug nào cũng hợp lệ
 * (dynamicParams mặc định true). Slug không tồn tại bị Proxy chặn trước (src/proxy.ts).
 * Bật song ngữ (NEXT_PUBLIC_ENABLE_I18N) ⇒ máy chủ đọc cookie ngôn ngữ ⇒ trang phải động: bỏ generateStaticParams.
 */
export const revalidate = 60;
export const generateStaticParams = I18N_ENABLED ? undefined : async (): Promise<{ slug: string }[]> => [];

const stickyAside = 'tkh-reader-scroll space-y-8';
const articleGrid = 'tkh-reader-grid';

const articlePath = (slug: string) => `/articles/${encodeURIComponent(slug)}`;

/** Ảnh chia sẻ: ảnh SEO riêng › ảnh bìa › ảnh sinh tự động từ tiêu đề bài. */
function shareImage(article: ArticleDetail): { url: string; generated: boolean } {
  const custom = article.seo?.ogImageUrl || articleCover(article);
  if (custom) return { url: absoluteUrl(custom), generated: false };
  return { url: absoluteUrl(`/og/articles/${encodeURIComponent(article.slug)}`), generated: true };
}

/** Số từ của phần chữ trong bài (bỏ khối mã, mã nội dòng, thẻ HTML, URL, ký hiệu Markdown) — cho JSON-LD wordCount. */
function countWords(markdown: string): number {
  const text = markdown
    .replace(/```[\s\S]*?```|~~~[\s\S]*?~~~/g, ' ')
    .replace(/`[^`\n]*`/g, ' ')
    .replace(/<[^>]+>/g, ' ')
    .replace(/\]\([^)]*\)/g, ' ')
    .replace(/https?:\/\/\S+/g, ' ')
    .replace(/[#>*_~|=[\]()!-]+/g, ' ');
  return text.split(/\s+/).filter((w) => /[\p{L}\p{N}]/u.test(w)).length;
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

  const share = shareImage(article);
  const shareImageObject = share.generated
    ? { '@type': 'ImageObject', url: share.url, width: 1200, height: 630 }
    : { '@type': 'ImageObject', url: share.url };

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
    image: [shareImageObject],
    mainEntityOfPage: { '@type': 'WebPage', '@id': url },
    url,
    // Ngôn ngữ NỘI DUNG bài (giao diện luôn tiếng Việt); bài không ghi ngôn ngữ ⇒ vi.
    inLanguage: article.language === 'en' ? 'en' : 'vi',
    articleSection: categoryName,
    keywords: article.tags.join(', '),
    wordCount: countWords(article.content),
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
      {/* Bản cho trình duyệt KHÔNG kèm `content` (Markdown thô): thân bài đã render ở máy chủ — gửi kèm là nhân đôi
          dung lượng payload RSC trong HTML. Không thành phần client nào đọc `content`. */}
      <ArticleProvider key={article.slug} initial={{ ...article, content: '' }}>
      <div className={articleGrid}>
        <aside className="tkh-reader-left" aria-label={path ? t('article.pathNav') : t('article.toc')}>
          <div className={stickyAside}>{path ? <PathNav path={path} currentSlug={article.slug} /> : <TocPanel headings={headings} />}</div>
        </aside>

        <article className="mx-auto w-full max-w-[820px] min-w-0">
          <ArticleHeader />

          {(path || headings.length > 0) && (
            <div className="mt-6 space-y-3 min-[1280px]:hidden">
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

          <ArticleBody>
            <ArticleContent markdown={article.content} />
          </ArticleBody>

          <div className="mt-10">
            <TagsBlock tags={article.tags} headingId="tags-title-inline" />
          </div>

          <div className="mt-12 space-y-10">
            <ReactionCards />
            <PrevNext article={article} />
            <div>
              <RelatedArticles articles={relatedArticles} headingId="related-title-inline" />
            </div>
            <LazyComments slug={article.slug} />
          </div>
        </article>

        <aside className="tkh-reader-right" aria-label={t('article.toc')}>
          <div className={stickyAside}>
            {headings.length > 0 && (
              <div className={path ? '' : 'min-[1600px]:hidden'}>
                <TocPanel headings={headings} />
              </div>
            )}
            {path && (
              <div className="min-[1600px]:hidden">
                <PathNav path={path} currentSlug={article.slug} defaultOpen={false} />
              </div>
            )}
          </div>
        </aside>
      </div>
      </ArticleProvider>
    </>
  );
}
