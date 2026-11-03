import { Link, useParams } from 'react-router';
import { useTranslation } from 'react-i18next';
import { useQuery } from '@tanstack/react-query';
import { ArrowRight, BookOpen, Check, Clock } from 'lucide-react';
import { Skeleton } from '@tkh/ui';
import { api, formatDate } from '@tkh/shared';
import { useLocale } from '../i18n';
import { ArticleMeta, articleHref } from '../components/common/ArticleCard';
import { CategoryChip, LevelTag } from '../components/common/Badges';
import { ProgressBar } from '../components/common/PathCard';
import { EmptyState, ErrorState, LoadingRegion } from '../components/common/States';
import { useDocumentTitle } from '../hooks/useDocumentTitle';
import { isNotFound } from '../lib/errors';
import { formatDuration } from '../lib/format';
import { useReadArticles } from '../lib/reading';
import { btnPrimary, card, container } from '../lib/ui';
import { NotFoundPage } from './NotFoundPage';

export function LearningPathPage() {
  const { slug = '' } = useParams();
  const { t } = useTranslation();
  const locale = useLocale();
  const read = useReadArticles();
  const path = useQuery({ queryKey: ['learning-path', slug], queryFn: () => api.learningPaths.get(slug) });
  useDocumentTitle(path.data?.title);

  if (path.isError && isNotFound(path.error)) return <NotFoundPage message={t('paths.notFound')} />;

  if (path.isPending) {
    return (
      <LoadingRegion className={`${container} max-w-4xl space-y-4 py-10`}>
        <Skeleton className="h-5 w-32" />
        <Skeleton className="h-10 w-3/4" />
        <Skeleton className="h-4 w-full" />
        <Skeleton className="h-2 w-full" />
        {[0, 1, 2, 3].map((i) => (
          <Skeleton key={i} className="h-20 w-full rounded-card" />
        ))}
      </LoadingRegion>
    );
  }
  if (path.isError) {
    return (
      <div className={`${container} max-w-4xl py-10`}>
        <ErrorState onRetry={() => void path.refetch()} />
      </div>
    );
  }

  const p = path.data;
  const doneCount = p.articles.filter((a) => read[a.id]).length;
  const nextArticle = p.articles.find((a) => !read[a.id]);
  const cta = nextArticle ?? p.articles[0];
  const ctaLabel = doneCount === 0 ? t('paths.start') : nextArticle ? t('paths.continue') : t('paths.review');

  return (
    <div className={`${container} max-w-4xl`}>
      <header className="space-y-4 py-10">
        <div className="flex flex-wrap items-center gap-2">
          <LevelTag level={p.level} />
          {p.category && <CategoryChip category={p.category} />}
        </div>
        <h1 className="text-3xl font-extrabold tracking-tight text-ink sm:text-4xl">{p.title}</h1>
        <p className="max-w-3xl text-muted sm:text-lg">{p.description}</p>
        <dl className="flex flex-wrap gap-x-6 gap-y-2 text-sm">
          <div className="flex items-center gap-1.5">
            <dt className="sr-only">{t('paths.articles')}</dt>
            <BookOpen aria-hidden="true" className="h-4 w-4 text-muted" />
            <dd className="font-semibold text-ink">{t('common.lessonCount', { count: p.articles.length, n: p.articles.length })}</dd>
          </div>
          <div className="flex items-center gap-1.5">
            <dt className="sr-only">{t('paths.totalTime')}</dt>
            <Clock aria-hidden="true" className="h-4 w-4 text-muted" />
            <dd className="font-semibold text-ink">{formatDuration(p.totalMinutes, t)}</dd>
          </div>
        </dl>
        <p className="text-sm text-muted">{t('common.updatedOn', { date: formatDate(p.updatedAt, locale) })}</p>
        {p.articles.length > 0 && (
          <div className={`flex flex-col gap-4 p-4 sm:flex-row sm:items-center ${card}`}>
            <div className="min-w-0 flex-1 space-y-2">
              <ProgressBar value={doneCount} max={p.articles.length} label={t('paths.progressLabel', { title: p.title })} />
              <p className="text-sm text-muted">{t('paths.progress', { read: doneCount, total: p.articles.length })}</p>
            </div>
            {cta && (
              <Link to={articleHref(cta.slug)} className={btnPrimary}>
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
                    <h3 className="font-semibold text-ink">
                      <Link
                        to={articleHref(a.slug)}
                        lang={a.language}
                        className="after:absolute after:inset-0 after:content-[''] hover:text-primary-ink focus-visible:outline-none focus-visible:after:rounded-card focus-visible:after:ring-2 focus-visible:after:ring-primary"
                      >
                        <span className="sr-only">{i + 1}. </span>
                        {a.title}
                      </Link>
                    </h3>
                    <p className="mt-1 line-clamp-2 text-sm text-muted" lang={a.language}>
                      {a.summary}
                    </p>
                    <div className="mt-2 flex flex-wrap items-center gap-3">
                      <ArticleMeta article={a} />
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
