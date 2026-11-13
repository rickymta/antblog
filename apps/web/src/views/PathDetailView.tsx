'use client';

import Link from 'next/link';
import { useTranslation } from 'react-i18next';
import { ArrowRight, BookOpen, Check, Clock } from 'lucide-react';
import type { LearningPathDetail } from '@tkh/shared';
import { useLocale } from '../i18n';
import { ArticleMeta, articleHref } from '../components/common/ArticleCard';
import { CategoryChip, LevelTag } from '../components/common/Badges';
import { ProgressBar } from '../components/common/PathCard';
import { EmptyState } from '../components/common/States';
import { formatDay } from '../lib/dates';
import { formatDuration } from '../lib/format';
import { useReadArticles } from '../lib/reading';
import { btnPrimary, card, container } from '../lib/ui';

/** Chi tiết lộ trình: dữ liệu do máy chủ tải; tiến độ đọc (localStorage) hiện sau khi hydrate. */
export function PathDetailView({ path: p }: { path: LearningPathDetail }) {
  const { t } = useTranslation();
  const locale = useLocale();
  const read = useReadArticles();

  const doneCount = p.articles.filter((a) => read[a.id]).length;
  const nextArticle = p.articles.find((a) => !read[a.id]);
  const cta = nextArticle ?? p.articles[0];
  const ctaLabel = doneCount === 0 ? t('paths.start') : nextArticle ? t('paths.continue') : t('paths.review');

  return (
    <div className={`${container} max-w-4xl`}>
      <nav aria-label={t('article.breadcrumb')} className="pt-6 text-sm"><Link href="/learning-paths" className="inline-flex min-h-11 items-center text-primary-ink hover:underline">← {t('paths.backToPaths')}</Link></nav>
      <header className="space-y-4 pt-4 pb-8">
        <div className="flex flex-wrap items-center gap-2">
          <LevelTag level={p.level} />
          {p.category && <CategoryChip category={p.category} />}
        </div>
        <h1 className="text-3xl font-extrabold tracking-tight text-ink sm:text-4xl">{p.title}</h1>
        <p className="max-w-3xl text-base leading-relaxed text-muted sm:text-lg">{p.description}</p>
        <dl className="flex flex-wrap gap-x-6 gap-y-2 text-sm">
          <div className="flex items-center gap-1.5">
            <dt className="sr-only">{t('paths.articles')}</dt>
            <BookOpen aria-hidden="true" className="h-4 w-4 text-muted" />
            <dd className="font-semibold leading-relaxed text-ink">{t('common.lessonCount', { count: p.articles.length, n: p.articles.length })}</dd>
          </div>
          <div className="flex items-center gap-1.5">
            <dt className="sr-only">{t('paths.totalTime')}</dt>
            <Clock aria-hidden="true" className="h-4 w-4 text-muted" />
            <dd className="font-semibold leading-relaxed text-ink">{formatDuration(p.totalMinutes, t)}</dd>
          </div>
        </dl>
        <p className="text-sm text-muted">{t('common.updatedOn', { date: formatDay(p.updatedAt, locale) })}</p>
        {p.articles.length > 0 && (
          <div className={`flex flex-col gap-4 p-4 sm:flex-row sm:items-center ${card}`}>
            <div className="min-w-0 flex-1 space-y-2">
              <ProgressBar value={doneCount} max={p.articles.length} label={t('paths.progressLabel', { title: p.title })} />
              <p className="text-sm text-muted">{t('paths.progress', { read: doneCount, total: p.articles.length })}</p>
              <p className="text-sm leading-relaxed text-muted">{t('paths.savedLocally')}</p>
              {nextArticle && <p className="text-sm leading-relaxed text-ink">{t('paths.nextLesson')}: <span className="font-semibold">{nextArticle.title}</span></p>}
            </div>
            {cta && (
              <Link href={articleHref(cta.slug)} className={btnPrimary}>
                {ctaLabel}
                <ArrowRight aria-hidden="true" className="h-4 w-4" />
              </Link>
            )}
          </div>
        )}
      </header>

      <section aria-labelledby="path-articles">
        <h2 id="path-articles" className="mb-4 text-xl font-bold text-ink">
          {t('paths.articles')}
        </h2>
        <p className="mb-6 max-w-3xl leading-relaxed text-muted">{t('paths.readGuide')}</p>
        {p.articles.length === 0 ? (
          <EmptyState title={t('paths.emptyArticles')} />
        ) : (
          <ol className="space-y-3">
            {p.articles.map((a, i) => {
              const done = Boolean(read[a.id]);
              return (
                <li key={a.id} className={`group relative flex gap-4 p-4 transition-colors hover:border-primary ${card}`}>
                  <span
                    className={`inline-flex h-9 w-9 shrink-0 items-center justify-center rounded-full text-sm font-bold ${
                      done ? 'bg-emerald-600 text-white' : 'bg-primary-soft text-primary-ink'
                    }`}
                  >
                    {done ? <Check aria-hidden="true" className="h-4 w-4" /> : i + 1}
                  </span>
                  <div className="min-w-0 flex-1">
                    <h3 className="font-semibold leading-relaxed text-ink">
                      <Link
                        href={articleHref(a.slug)}
                        lang={a.language}
                        className="after:absolute after:inset-0 after:content-[''] hover:text-primary-ink focus-visible:outline-none focus-visible:after:rounded-card focus-visible:after:ring-2 focus-visible:after:ring-primary"
                      >
                        <span className="sr-only">{i + 1}. </span>
                        {a.title}
                      </Link>
                    </h3>
                    <p className="mt-1 text-sm leading-relaxed text-muted" lang={a.language}>
                      {a.summary}
                    </p>
                    <div className="mt-2 flex flex-wrap items-center gap-3">
                      <ArticleMeta article={a} />
                      {a.id === nextArticle?.id && <span className="text-sm font-semibold text-primary-ink">{t('paths.nextLesson')}</span>}
                      {done && <span className="text-xs font-semibold text-emerald-700 dark:text-emerald-400">✓ {t('paths.done')}</span>}
                    </div>
                  </div>
                </li>
              );
            })}
          </ol>
        )}
      </section>
    </div>
  );
}
