'use client';

import Link from 'next/link';
import { useTranslation } from 'react-i18next';
import { BookOpen, Calendar, Clock } from 'lucide-react';
import { Skeleton } from '@tkh/ui';
import { type LearningPathCard } from '@tkh/shared';
import { formatDay } from '../../lib/dates';
import { useLocale } from '../../i18n';
import { formatDuration } from '../../lib/format';
import { countReadInPath, useReadArticles } from '../../lib/reading';
import { btnPrimary, card } from '../../lib/ui';
import { CategoryChip, LevelTag } from './Badges';

export const pathHref = (slug: string) => `/learning-paths/${encodeURIComponent(slug)}`;

export function ProgressBar({ value, max, label }: { value: number; max: number; label: string }) {
  const pct = max > 0 ? Math.round((value / max) * 100) : 0;
  return (
    <div
      role="progressbar"
      aria-label={label}
      aria-valuemin={0}
      aria-valuemax={max}
      aria-valuenow={value}
      className="h-2 w-full overflow-hidden rounded-full bg-slate-200 dark:bg-slate-700"
    >
      <div className="h-full rounded-full bg-gradient-to-r from-primary to-secondary transition-[width]" style={{ width: `${pct}%` }} />
    </div>
  );
}

/**
 * Thẻ lộ trình (A6): nhãn chủ đề chữ hoa + nhãn cấp độ, tiêu đề lớn, mô tả, số bài + thời lượng / ngày,
 * thanh tiến độ (đã đọc trên máy này) và nút rộng hết thẻ dính đáy.
 */
export function PathCard({ path, showDate = false, headingLevel = 3 }: { path: LearningPathCard; showDate?: boolean; headingLevel?: 2 | 3 }) {
  const { t } = useTranslation();
  const locale = useLocale();
  const Heading = headingLevel === 2 ? 'h2' : 'h3';
  const read = countReadInPath(useReadArticles(), path.slug, path.articleCount);

  return (
    <article className={`flex h-full flex-col gap-4 p-5 ${card}`}>
      <div className="flex flex-wrap items-start justify-between gap-2">
        {path.category ? <CategoryChip category={path.category} uppercase /> : <span />}
        <LevelTag level={path.level} />
      </div>
      <div className="space-y-2">
        <Heading className="text-lg leading-snug font-bold text-ink"><Link href={pathHref(path.slug)} className="hover:text-primary-ink hover:underline">{path.title}</Link></Heading>
        <p className="text-base leading-relaxed text-muted">{path.description}</p>
      </div>
      <div className="mt-auto space-y-3">
        <div className="flex flex-wrap items-center gap-x-4 gap-y-1 text-sm text-muted">
          <span className="inline-flex items-center gap-1">
            <BookOpen aria-hidden="true" className="h-3.5 w-3.5" />
            {t('common.lessonCount', { count: path.articleCount, n: path.articleCount })}
          </span>
          <span className="inline-flex items-center gap-1">
            <Clock aria-hidden="true" className="h-3.5 w-3.5" />
            {formatDuration(path.totalMinutes, t)}
          </span>
          {showDate && (
            <span className="inline-flex items-center gap-1">
              <Calendar aria-hidden="true" className="h-3.5 w-3.5" />
              {t('common.updatedOn', { date: formatDay(path.updatedAt, locale) })}
            </span>
          )}
        </div>
        {path.articleCount > 0 ? <div className="space-y-1.5">
          <ProgressBar value={read} max={path.articleCount} label={t('paths.progressLabel', { title: path.title })} />
          <p className="text-sm text-muted">{t('paths.progress', { read, total: path.articleCount })}</p>
        </div> : <p className="text-sm text-muted">{t('paths.emptyArticles')}</p>}
        <Link href={pathHref(path.slug)} className={`${btnPrimary} w-full`}>
          {t('paths.view')}
          <span className="sr-only">: {path.title}</span>
        </Link>
      </div>
    </article>
  );
}

export function PathCardSkeleton() {
  return (
    <div className={`flex flex-col gap-4 p-5 ${card}`}>
      <div className="flex justify-between">
        <Skeleton className="h-5 w-24" />
        <Skeleton className="h-5 w-16" />
      </div>
      <Skeleton className="h-6 w-3/4" />
      <Skeleton className="h-4 w-full" />
      <Skeleton className="h-4 w-5/6" />
      <Skeleton className="mt-4 h-2 w-full" />
      <Skeleton className="h-9 w-full" />
    </div>
  );
}
