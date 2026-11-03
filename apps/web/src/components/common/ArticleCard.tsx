import { Link } from 'react-router';
import { useTranslation } from 'react-i18next';
import { Clock, Eye, Heart } from 'lucide-react';
import { Skeleton } from '@tkh/ui';
import { formatCompact, formatDate, type ArticleCard as ArticleCardData } from '@tkh/shared';
import { useLocale } from '../../i18n';
import { totalReactions } from '../../lib/format';
import { card } from '../../lib/ui';
import { CategoryChip, LevelTag } from './Badges';
import { BookmarkButton } from './BookmarkButton';
import { Cover } from './Cover';

export const articleHref = (slug: string) => `/articles/${encodeURIComponent(slug)}`;

/** Chỉ số ⏱ phút đọc · 👁 lượt xem — dùng ở thẻ và dòng bài. */
export function ArticleMeta({ article, className = '' }: { article: ArticleCardData; className?: string }) {
  const { t } = useTranslation();
  const locale = useLocale();
  return (
    <span className={`inline-flex flex-wrap items-center gap-x-3 gap-y-1 text-xs text-muted ${className}`}>
      <span className="inline-flex items-center gap-1">
        <Clock aria-hidden="true" className="h-3.5 w-3.5" />
        {t('common.minRead', { count: article.readingMinutes })}
      </span>
      <span className="inline-flex items-center gap-1" title={t('common.views', { n: formatCompact(article.stats.views, locale) })}>
        <Eye aria-hidden="true" className="h-3.5 w-3.5" />
        <span aria-hidden="true">{formatCompact(article.stats.views, locale)}</span>
        <span className="sr-only">{t('common.views', { n: formatCompact(article.stats.views, locale) })}</span>
      </span>
    </span>
  );
}

/**
 * Thẻ bài viết (lưới): ảnh bìa hoặc khối màu chủ đề, nhãn chủ đề + cấp độ, tiêu đề 2 dòng, phút đọc · lượt xem, ❤ tổng reaction, nút lưu.
 * Cả thẻ bấm được nhờ liên kết tiêu đề phủ toàn thẻ; nhãn chủ đề và nút lưu nằm lớp trên.
 */
export function ArticleCard({ article, showSummary = false, headingLevel = 3 }: { article: ArticleCardData; showSummary?: boolean; headingLevel?: 2 | 3 }) {
  const { t } = useTranslation();
  const locale = useLocale();
  const Heading = headingLevel === 2 ? 'h2' : 'h3';
  const reactions = totalReactions(article.stats.reactions);

  return (
    <article className={`group relative flex h-full flex-col overflow-hidden ${card} transition-shadow hover:shadow-lg hover:shadow-slate-900/5 dark:hover:shadow-black/30`}>
      <div className="aspect-[16/9] overflow-hidden">
        <Cover src={article.coverImageUrl} category={article.category} />
      </div>
      <div className="flex flex-1 flex-col gap-3 p-4">
        <div className="relative z-10 flex flex-wrap items-center gap-2">
          <CategoryChip category={article.category} />
          <LevelTag level={article.level} />
        </div>
        <Heading className="line-clamp-2 text-base leading-snug font-semibold text-ink">
          <Link
            to={articleHref(article.slug)}
            lang={article.language}
            className="after:absolute after:inset-0 after:content-[''] hover:text-primary-ink focus-visible:outline-none focus-visible:after:rounded-card focus-visible:after:ring-2 focus-visible:after:ring-primary"
          >
            {article.title}
          </Link>
        </Heading>
        {showSummary && (
          <p className="line-clamp-2 text-sm text-muted" lang={article.language}>
            {article.summary}
          </p>
        )}
        <div className="mt-auto flex items-center justify-between gap-2 pt-1">
          <ArticleMeta article={article} />
          <div className="relative z-10 flex items-center gap-1">
            <span className="inline-flex items-center gap-1 text-xs text-muted" title={t('common.reactionsTotal', { n: formatCompact(reactions, locale) })}>
              <Heart aria-hidden="true" className="h-3.5 w-3.5 text-rose-500" />
              <span aria-hidden="true">{formatCompact(reactions, locale)}</span>
              <span className="sr-only">{t('common.reactionsTotal', { n: formatCompact(reactions, locale) })}</span>
            </span>
            <BookmarkButton article={article} />
          </div>
        </div>
        <span className="sr-only">{t('common.publishedOn', { date: formatDate(article.publishedAt, locale) })}</span>
      </div>
    </article>
  );
}

export function ArticleCardSkeleton() {
  return (
    <div className={`flex flex-col overflow-hidden ${card}`}>
      <Skeleton className="aspect-[16/9] rounded-none" />
      <div className="flex flex-col gap-3 p-4">
        <div className="flex gap-2">
          <Skeleton className="h-5 w-20" />
          <Skeleton className="h-5 w-16" />
        </div>
        <Skeleton className="h-5 w-full" />
        <Skeleton className="h-5 w-2/3" />
        <Skeleton className="mt-2 h-4 w-1/2" />
      </div>
    </div>
  );
}

/** Dòng bài gọn (bài mới nhất, bài liên quan): ảnh nhỏ + tiêu đề + phút đọc. */
export function ArticleRow({ article, showMeta = true, compact = false }: { article: ArticleCardData; showMeta?: boolean; compact?: boolean }) {
  const locale = useLocale();
  return (
    <article className="group relative flex items-start gap-3">
      <div className={`shrink-0 overflow-hidden rounded-ui ${compact ? 'h-12 w-16' : 'h-16 w-24 sm:h-20 sm:w-32'}`}>
        <Cover src={article.coverImageUrl} category={article.category} iconSize={compact ? 'h-5 w-5' : 'h-7 w-7'} />
      </div>
      <div className="min-w-0 flex-1">
        {!compact && (
          <div className="relative z-10 mb-1 flex flex-wrap items-center gap-2">
            <CategoryChip category={article.category} />
            <LevelTag level={article.level} />
          </div>
        )}
        <h3 className={`line-clamp-2 leading-snug font-semibold text-ink ${compact ? 'text-sm' : 'text-base'}`}>
          <Link
            to={articleHref(article.slug)}
            lang={article.language}
            className="after:absolute after:inset-0 after:content-[''] hover:text-primary-ink focus-visible:outline-none focus-visible:after:rounded-ui focus-visible:after:ring-2 focus-visible:after:ring-primary"
          >
            {article.title}
          </Link>
        </h3>
        {showMeta && (
          <div className="mt-1 flex flex-wrap items-center gap-x-3 text-xs text-muted">
            <ArticleMeta article={article} />
            {!compact && <span>{formatDate(article.publishedAt, locale)}</span>}
          </div>
        )}
      </div>
    </article>
  );
}

export function ArticleRowSkeleton({ compact = false }: { compact?: boolean }) {
  return (
    <div className="flex items-start gap-3">
      <Skeleton className={compact ? 'h-12 w-16' : 'h-16 w-24 sm:h-20 sm:w-32'} />
      <div className="flex-1 space-y-2">
        <Skeleton className="h-4 w-full" />
        <Skeleton className="h-4 w-1/2" />
      </div>
    </div>
  );
}
