'use client';

import Link from 'next/link';
import { useTranslation } from 'react-i18next';
import { ArrowRight, BookOpen, CircleCheck, Clock } from 'lucide-react';
import type { LearningPathDetail } from '@tkh/shared';
import { formatDuration } from '../../lib/format';
import { useReadArticles } from '../../lib/reading';
import { btnLink, card } from '../../lib/ui';
import { articleHref } from '../common/ArticleCard';
import { LevelTag } from '../common/Badges';
import { pathHref } from '../common/PathCard';

const two = (n: number) => String(n).padStart(2, '0');

/**
 * Trang chủ đề đọc theo từng phần: cột trái là mục lục các phần (lộ trình của chủ đề, đúng thứ tự học),
 * cột phải liệt kê bài của từng phần theo thứ tự đọc. Dấu "đã đọc" lấy từ trình duyệt này.
 */
export function TopicParts({ parts }: { parts: LearningPathDetail[] }) {
  const { t } = useTranslation();
  const read = useReadArticles();
  const readCount = (part: LearningPathDetail) => part.articles.filter(a => read[a.id]).length;

  return (
    <section aria-labelledby="topic-parts" className="pb-12">
      <div className="mb-6 max-w-3xl space-y-2">
        <h2 id="topic-parts" className="text-2xl font-bold text-ink">{t('topic.partsTitle')}</h2>
        <p className="text-base leading-relaxed text-muted">{t('topic.partsGuide')}</p>
      </div>
      <div className="grid gap-8 lg:grid-cols-[18rem_minmax(0,1fr)] lg:gap-10">
        <nav aria-label={t('topic.partsNav')} className={`${card} p-3 lg:sticky lg:top-24 lg:max-h-[calc(100dvh-7rem)] lg:self-start lg:overflow-y-auto`}>
          <p className="px-2 pt-1 pb-2 text-sm font-semibold text-ink">{t('topic.partsNav')} ({parts.length})</p>
          <ol className="space-y-1">
            {parts.map((part, i) => {
              const done = readCount(part);
              return <li key={part.id}>
                <a href={`#part-${part.slug}`} className="flex min-h-11 gap-3 rounded-ui px-2 py-2 hover:bg-surface-2">
                  <span aria-hidden="true" className="pt-0.5 font-mono text-sm text-primary-ink">{two(i + 1)}</span>
                  <span className="min-w-0">
                    <span className="block text-sm leading-snug font-medium text-ink">{part.title}</span>
                    <span className="mt-1 block text-sm text-muted">
                      {done > 0 ? t('paths.progress', { read: done, total: part.articles.length }) : t('common.lessonCount', { count: part.articles.length, n: part.articles.length })}
                    </span>
                  </span>
                </a>
              </li>;
            })}
          </ol>
        </nav>

        <div className="min-w-0 space-y-8">
          {parts.map((part, i) => <section key={part.id} id={`part-${part.slug}`} aria-labelledby={`part-title-${part.slug}`} className={`${card} scroll-mt-28 overflow-hidden`}>
            <header className="space-y-3 p-5 sm:p-6">
              <div className="flex flex-wrap items-center justify-between gap-2">
                <p className="font-mono text-sm tracking-wider text-primary-ink uppercase">{t('topic.partLabel', { n: two(i + 1) })}</p>
                <LevelTag level={part.level} />
              </div>
              <h3 id={`part-title-${part.slug}`} className="text-xl leading-snug font-bold text-ink">{part.title}</h3>
              {part.description && <p className="max-w-prose text-base leading-relaxed text-muted">{part.description}</p>}
              <div className="flex flex-wrap items-center gap-x-5 gap-y-1 text-sm text-muted">
                <span className="inline-flex items-center gap-1.5"><BookOpen aria-hidden="true" className="h-4 w-4" />{t('common.lessonCount', { count: part.articles.length, n: part.articles.length })}</span>
                <span className="inline-flex items-center gap-1.5"><Clock aria-hidden="true" className="h-4 w-4" />{formatDuration(part.totalMinutes, t)}</span>
                <Link href={pathHref(part.slug)} className={`${btnLink} min-h-11 sm:ml-auto`}>
                  {t('paths.view')}<span className="sr-only">: {part.title}</span><ArrowRight aria-hidden="true" className="h-4 w-4 shrink-0" />
                </Link>
              </div>
            </header>
            <ol className="divide-y divide-line border-t border-line">
              {part.articles.map((a, n) => {
                const isRead = Boolean(read[a.id]);
                return <li key={a.id}>
                  <Link href={articleHref(a.slug)} className="group flex min-h-14 items-baseline gap-4 px-5 py-3 hover:bg-surface-2 sm:px-6">
                    <span aria-hidden="true" className="w-6 shrink-0 font-mono text-sm text-muted">{two(n + 1)}</span>
                    <span className="min-w-0 flex-1 text-base leading-snug font-medium text-ink group-hover:text-primary-ink">{a.title}</span>
                    <span className="flex shrink-0 items-center gap-2 self-center text-sm text-muted">
                      {isRead && <CircleCheck role="img" aria-label={t('topic.read')} className="h-4 w-4 text-emerald-600 dark:text-emerald-400" />}
                      <span className="hidden sm:inline">{t('common.minRead', { count: a.readingMinutes })}</span>
                    </span>
                  </Link>
                </li>;
              })}
            </ol>
          </section>)}
        </div>
      </div>
    </section>
  );
}
