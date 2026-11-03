import { useEffect, useMemo, useRef } from 'react';
import { useLocation, useParams } from 'react-router';
import { useTranslation } from 'react-i18next';
import { useQuery } from '@tanstack/react-query';
import { ArticleContent, extractHeadings, Skeleton } from '@tkh/ui';
import { api, type ArticleDetail } from '@tkh/shared';
import { PrevNext, RelatedArticles, ShareBlock, TagsBlock } from '../components/article/Aside';
import { ArticleHeader } from '../components/article/ArticleHeader';
import { CodeCopyOverlay } from '../components/article/CodeCopyOverlay';
import { PathNav } from '../components/article/PathNav';
import { ReactionCards } from '../components/article/ReactionCards';
import { TocCollapsible, TocPanel } from '../components/article/Toc';
import { CommentsSection } from '../components/comments/CommentsSection';
import { ErrorState, LoadingRegion } from '../components/common/States';
import { useArticleView } from '../hooks/useArticleView';
import { useDocumentTitle } from '../hooks/useDocumentTitle';
import { useScrollSpy } from '../hooks/useScrollSpy';
import { isNotFound } from '../lib/errors';
import { card, container } from '../lib/ui';
import { NotFoundPage } from './NotFoundPage';

const stickyAside = 'sticky top-24 max-h-[calc(100dvh-7rem)] space-y-8 overflow-y-auto overscroll-contain pr-1 pb-4';
// Trang bài rộng hơn khung chung (max-w-7xl) để cột giữa đạt 760px ở màn ≥ 1400px — hình SVG trong bài cần ~720px mới khỏi cuộn ngang.
const articleGrid = `${container} min-[1400px]:max-w-[1440px] grid gap-10 py-8 lg:grid-cols-[minmax(0,1fr)_16rem] xl:grid-cols-[15rem_minmax(0,1fr)_16rem]`;

/**
 * Trang bài viết (docs/ui-guide.md §6):
 * ≥ 1280px 3 cột — trái: danh sách bài của lộ trình (hoặc mục lục khi bài không thuộc lộ trình); giữa: nội dung; phải: "Trên trang này", bài liên quan, tags, chia sẻ.
 * 1024–1279px: bỏ cột trái, mục lục + lộ trình lên cột phải. < 1024px: một cột, mục lục thu gọn đầu bài.
 */
function ArticleView({ article }: { article: ArticleDetail }) {
  const { t } = useTranslation();
  const location = useLocation();
  const bodyRef = useRef<HTMLDivElement>(null);
  const headings = useMemo(() => extractHeadings(article.content), [article.content]);
  const ids = useMemo(() => headings.map((h) => h.id), [headings]);
  const activeId = useScrollSpy(ids);
  const path = article.learningPath;

  useArticleView(article, bodyRef);

  // Liên kết có #mục (chia sẻ mục lục) ⇒ cuộn tới khi nội dung đã render.
  useEffect(() => {
    if (!location.hash) return;
    const id = decodeURIComponent(location.hash.slice(1));
    const frame = window.requestAnimationFrame(() => document.getElementById(id)?.scrollIntoView());
    return () => window.cancelAnimationFrame(frame);
    // Chỉ lúc mở bài.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return (
    <div className={articleGrid}>
      <aside className="hidden xl:block" aria-label={path ? t('article.pathNav') : t('article.toc')}>
        <div className={stickyAside}>
          {path ? <PathNav path={path} currentSlug={article.slug} /> : <TocPanel headings={headings} activeId={activeId} />}
        </div>
      </aside>

      <article className="mx-auto w-full max-w-[760px] min-w-0">
        <ArticleHeader article={article} />

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
            <TocCollapsible headings={headings} activeId={activeId} />
          </div>
        )}

        {article.coverImageUrl && (
          <div className="mt-8 aspect-[16/9] overflow-hidden rounded-card border border-line bg-surface-2">
            <img src={article.coverImageUrl} alt="" className="h-full w-full object-cover" referrerPolicy="no-referrer" />
          </div>
        )}

        <div ref={bodyRef} lang={article.language} className="tkh-article-body relative mt-8">
          <ArticleContent markdown={article.content} />
          <CodeCopyOverlay containerRef={bodyRef} contentKey={article.id + article.updatedAt} />
        </div>

        <div className="mt-10 lg:hidden">
          <TagsBlock tags={article.tags} />
        </div>

        <div className="mt-12 space-y-10">
          <ReactionCards article={article} />
          <PrevNext article={article} />
          <div className="lg:hidden">
            <RelatedArticles slug={article.slug} />
          </div>
          <CommentsSection slug={article.slug} />
        </div>
      </article>

      <aside className="hidden lg:block" aria-label={t('article.toc')}>
        <div className={stickyAside}>
          {headings.length > 0 && (
            <div className={path ? '' : 'xl:hidden'}>
              <TocPanel headings={headings} activeId={activeId} />
            </div>
          )}
          {path && (
            <div className="xl:hidden">
              <PathNav path={path} currentSlug={article.slug} />
            </div>
          )}
          <RelatedArticles slug={article.slug} />
          <TagsBlock tags={article.tags} />
          <ShareBlock />
        </div>
      </aside>
    </div>
  );
}

function ArticleSkeleton() {
  return (
    <LoadingRegion className={articleGrid}>
      <div className="hidden space-y-3 xl:block">
        {[0, 1, 2, 3, 4].map((i) => (
          <Skeleton key={i} className="h-6 w-full" />
        ))}
      </div>
      <div className="mx-auto w-full max-w-[760px] space-y-5">
        <Skeleton className="h-4 w-1/2" />
        <Skeleton className="h-12 w-full" />
        <Skeleton className="h-12 w-3/4" />
        <Skeleton className="h-5 w-1/3" />
        <Skeleton className="mt-8 aspect-[16/9] w-full rounded-card" />
        {[0, 1, 2, 3, 4, 5].map((i) => (
          <Skeleton key={i} className="h-4 w-full" />
        ))}
      </div>
      <div className="hidden space-y-3 lg:block">
        {[0, 1, 2, 3].map((i) => (
          <Skeleton key={i} className="h-5 w-full" />
        ))}
      </div>
    </LoadingRegion>
  );
}

export function ArticlePage() {
  const { slug = '' } = useParams();
  const { t } = useTranslation();
  const article = useQuery({ queryKey: ['article', slug], queryFn: () => api.articles.get(slug) });
  useDocumentTitle(article.data?.title);

  // Có dữ liệu (kể cả khi lần tải lại sau đó lỗi) ⇒ vẫn hiển thị bài.
  // key theo slug: chuyển bài (bài trước/sau) dựng lại toàn bộ trạng thái cục bộ (bình luận, đếm lượt xem, nút sao chép mã).
  if (article.data) return <ArticleView key={article.data.slug} article={article.data} />;
  if (article.isPending) return <ArticleSkeleton />;
  if (isNotFound(article.error)) return <NotFoundPage message={t('article.notFound')} />;
  return (
    <div className={`${container} max-w-3xl py-16`}>
      <ErrorState onRetry={() => void article.refetch()} />
    </div>
  );
}
