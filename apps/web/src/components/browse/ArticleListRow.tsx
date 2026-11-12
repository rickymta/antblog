'use client';

import Link from 'next/link';
import { useTranslation } from 'react-i18next';
import { CircleCheck, Clock, Eye, Heart, Route } from 'lucide-react';
import { formatCompact, type ArticleCard as ArticleCardData } from '@tkh/shared';
import { useLocale } from '../../i18n';
import { articleCover } from '../../lib/article-cover';
import { formatDay } from '../../lib/dates';
import { totalReactions } from '../../lib/format';
import { useReadArticles } from '../../lib/reading';
import { articleHref } from '../common/ArticleCard';
import { CategoryChip, LevelTag } from '../common/Badges';
import { BookmarkButton } from '../common/BookmarkButton';
import { Cover } from '../common/Cover';
import { pathHref } from '../common/PathCard';

/**
 * Dòng bài trong danh sách đọc: ưu tiên thứ người học cần để chọn bài — thuộc phần nào, bài thứ mấy, cấp độ, thời lượng,
 * đã đọc chưa. Lượt xem / cảm xúc chỉ hiện khi khác 0. `showPath = false` khi danh sách đã có tiêu đề nhóm theo lộ trình.
 */
export function ArticleListRow({ article, showPath = true, headingLevel = 3, eagerCover = false }: {
  article: ArticleCardData;
  showPath?: boolean;
  headingLevel?: 3 | 4;
  eagerCover?: boolean;
}) {
  const { t } = useTranslation();
  const locale = useLocale();
  const Heading = headingLevel === 4 ? 'h4' : 'h3';
  const isRead = Boolean(useReadArticles()[article.id]);
  const reactions = totalReactions(article.stats.reactions);
  const ref = article.pathRef;

  return (
    <article className="group relative flex gap-4 px-4 py-4 hover:bg-surface-2 sm:gap-5 sm:px-5">
      <div className="hidden h-[5.5rem] w-36 shrink-0 overflow-hidden rounded-ui sm:block">
        <Cover src={articleCover(article)} category={article.category} eager={eagerCover} iconSize="h-7 w-7" />
      </div>
      <div className="min-w-0 flex-1 space-y-1.5">
        <div className="relative z-10 flex flex-wrap items-center gap-x-3 gap-y-1.5">
          {ref && <span className="font-mono text-sm text-primary-ink">{t('articles.lesson', { n: ref.position, total: ref.total })}</span>}
          <CategoryChip category={article.category} />
          <LevelTag level={article.level} />
        </div>
        <Heading className="text-lg leading-snug font-semibold text-ink">
          <Link
            href={articleHref(article.slug)}
            lang={article.language}
            className="after:absolute after:inset-0 after:content-[''] group-hover:text-primary-ink focus-visible:outline-none focus-visible:after:ring-2 focus-visible:after:ring-primary focus-visible:after:ring-inset"
          >
            {article.title}
          </Link>
        </Heading>
        <p className="line-clamp-2 max-w-prose text-base leading-relaxed text-muted" lang={article.language}>{article.summary}</p>
        <div className="flex flex-wrap items-center gap-x-4 gap-y-1 pt-0.5 text-sm text-muted">
          {isRead && <span className="inline-flex items-center gap-1 font-medium text-emerald-700 dark:text-emerald-400"><CircleCheck aria-hidden="true" className="h-4 w-4" />{t('topic.read')}</span>}
          <span className="inline-flex items-center gap-1"><Clock aria-hidden="true" className="h-4 w-4" />{t('common.minRead', { count: article.readingMinutes })}</span>
          <span>{formatDay(article.publishedAt, locale)}</span>
          {article.stats.views > 0 && <span className="inline-flex items-center gap-1"><Eye aria-hidden="true" className="h-4 w-4" />{t('common.views', { n: formatCompact(article.stats.views, locale) })}</span>}
          {reactions > 0 && <span className="inline-flex items-center gap-1"><Heart aria-hidden="true" className="h-4 w-4 text-rose-500" /><span aria-hidden="true">{formatCompact(reactions, locale)}</span><span className="sr-only">{t('common.reactionsTotal', { n: formatCompact(reactions, locale) })}</span></span>}
          {showPath && ref && <Link href={pathHref(ref.slug)} className="relative z-10 inline-flex min-w-0 items-center gap-1 text-primary-ink hover:underline">
            <Route aria-hidden="true" className="h-4 w-4 shrink-0" /><span className="truncate">{ref.title}</span>
          </Link>}
        </div>
      </div>
      <BookmarkButton article={article} className="relative z-10 h-11 w-11 shrink-0" />
    </article>
  );
}
