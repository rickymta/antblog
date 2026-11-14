'use client';

import { useEffect, useId, useRef, useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import Link from 'next/link';
import { useTranslation } from 'react-i18next';
import { X } from 'lucide-react';
import { api, type LearningPathCard } from '@tkh/shared';
import { formatDuration } from '../../lib/format';
import { useReadArticles } from '../../lib/reading';
import { btnPrimary, btnSecondary, card } from '../../lib/ui';
import { learningPathCover } from '../../lib/path-cover';
import { articleHref } from './ArticleCard';
import { Cover } from './Cover';

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

function PreviewLessons({ path }: { path: LearningPathCard }) {
  const { t } = useTranslation();
  const read = useReadArticles();
  const query = useQuery({
    queryKey: ['learning-path-preview', path.slug],
    queryFn: () => api.learningPaths.get(path.slug),
    staleTime: 5 * 60_000,
    retry: false,
  });
  const lessons = query.data?.articles ?? [];
  const preview = lessons.slice(0, 5);
  const unread = lessons.find(lesson => !read[lesson.id]);
  const target = unread ?? lessons[0];
  const hasRead = lessons.some(lesson => read[lesson.id]);
  const action = !hasRead ? 'paths.start' : unread ? 'paths.continue' : 'paths.review';
  const count = query.data ? lessons.length : path.articleCount;

  return (
    <>
      {query.isPending ? <p role="status">{t('common.loading')}</p> : query.isError && !query.data ? (
        <div>
          <p role="alert">{t('paths.previewError')}</p>
          <button type="button" className={`${btnSecondary} mt-2 min-h-11`} onClick={() => query.refetch()}>{t('common.retry')}</button>
        </div>
      ) : preview.length ? (
        <section>
          <h3 className="mb-2 font-semibold text-ink">{t('paths.previewLessons', { n: preview.length, total: count })}</h3>
          <ol className="list-decimal space-y-2 pl-6 text-ink">
            {preview.map(lesson => <li key={lesson.id} lang={lesson.language}>{lesson.title}</li>)}
          </ol>
        </section>
      ) : <p>{t('paths.emptyArticles')}</p>}
      <div className="flex flex-col gap-3 border-t border-line pt-4 sm:flex-row sm:flex-wrap">
        {target && <Link href={articleHref(target.slug)} className={`${btnPrimary} min-h-11`}>{t(action)}</Link>}
        <Link href={pathHref(path.slug)} className={`${btnSecondary} min-h-11`}>
          {count ? t('paths.previewAll', { n: count }) : t('paths.view')}
        </Link>
      </div>
    </>
  );
}

export function PathCard({ path }: { path: LearningPathCard }) {
  const { t } = useTranslation();
  const dialog = useRef<HTMLDialogElement>(null);
  const titleId = useId();
  const [open, setOpen] = useState(false);

  useEffect(() => {
    if (!open) return;
    const previous = document.body.style.overflow;
    document.body.style.overflow = 'hidden';
    return () => { document.body.style.overflow = previous; };
  }, [open]);

  return (
    <>
      <article className={`relative flex h-full flex-col overflow-hidden hover:border-primary ${card}`}>
        <div className="aspect-video overflow-hidden border-b border-line">
          <Cover src={learningPathCover(path.slug)} category={path.category ?? { name: { vi: path.title, en: path.title }, color: '#2563EB', icon: 'code' }} />
        </div>
        <div className="flex flex-1 flex-col gap-2 p-4">
          <h3 className="text-base leading-snug font-semibold text-ink">
            <Link href={pathHref(path.slug)} prefetch={false} aria-haspopup="dialog"
              className="after:absolute after:inset-0 hover:text-primary-ink focus-visible:after:rounded-card focus-visible:after:ring-2 focus-visible:after:ring-inset focus-visible:after:ring-primary"
              onClick={event => {
                if (event.metaKey || event.ctrlKey || event.shiftKey || event.altKey || !dialog.current) return;
                event.preventDefault();
                dialog.current.showModal();
                setOpen(true);
              }}>
              {path.title}
            </Link>
          </h3>
          <p className="mt-auto text-sm text-muted">{t(`common.levels.${path.level}`)} · {t('common.lessonCount', { count: path.articleCount, n: path.articleCount })}</p>
        </div>
      </article>
      <dialog ref={dialog} aria-labelledby={titleId} onClose={() => setOpen(false)}
        className="fixed inset-0 m-auto max-h-[90dvh] w-[calc(100%-2rem)] max-w-xl overflow-y-auto overscroll-contain rounded-card border border-line bg-surface p-0 text-ink shadow-xl backdrop:bg-black/50">
        <div className="flex items-start justify-between gap-4 border-b border-line p-5">
          <h2 id={titleId} className="text-xl leading-snug font-bold">{path.title}</h2>
          <button type="button" aria-label={t('common.close')} onClick={() => dialog.current?.close()} className="inline-flex h-11 w-11 shrink-0 items-center justify-center rounded-ui hover:bg-surface-2"><X aria-hidden="true" size={20} /></button>
        </div>
        {open && <div className="space-y-5 p-5 leading-relaxed text-muted">
          <p>{path.description}</p>
          <p className="text-sm">{t(`common.levels.${path.level}`)} · {t('paths.totalTime')}: {formatDuration(path.totalMinutes, t)}</p>
          <PreviewLessons path={path} />
        </div>}
      </dialog>
    </>
  );
}
