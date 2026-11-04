'use client';

import { useMemo, useState } from 'react';
import Link from 'next/link';
import { usePathname, useRouter } from 'next/navigation';
import { useTranslation } from 'react-i18next';
import { useInfiniteQuery, useQueryClient } from '@tanstack/react-query';
import { MessageSquare } from 'lucide-react';
import { Skeleton } from '@tkh/ui';
import { api, formatNumber, type Comment, type CommentSort, type CommentThread } from '@tkh/shared';
import { useLocale } from '../../i18n';
import { useAuth } from '../../lib/auth';
import { loginHref, registerHref } from '../../lib/navigation';
import { btnPrimary, btnSecondary } from '../../lib/ui';
import { ErrorState, LoadingRegion } from '../common/States';
import { CommentComposer } from './CommentComposer';
import { CommentItem } from './CommentItem';
import { appendReply, bumpArticleComments, commentsKey } from './commentCache';

const PAGE_SIZE = 10;
const COLLAPSE_AFTER = 2;
const SORTS: readonly CommentSort[] = ['newest', 'oldest', 'top'];

interface ReplyTarget {
  /** id gửi lên làm `parentId`: bình luận gốc, hoặc phản hồi được bấm "Trả lời" (API tự gắn về gốc + đặt `replyTo`). */
  parentId: string;
  name: string;
  prefill: string;
}

function Thread({ slug, thread, onRequireLogin }: { slug: string; thread: CommentThread; onRequireLogin: () => void }) {
  const { t } = useTranslation();
  const qc = useQueryClient();
  const { user } = useAuth();
  const [expanded, setExpanded] = useState(false);
  const [target, setTarget] = useState<ReplyTarget | null>(null);

  const replies = thread.replies;
  const hiddenCount = replies.length - COLLAPSE_AFTER;
  const collapsible = hiddenCount > 0;
  const shown = collapsible && !expanded ? replies.slice(0, COLLAPSE_AFTER) : replies;

  const startReply = (c: Comment) => {
    const isRoot = c.id === thread.id;
    setTarget({ parentId: c.id, name: c.author.displayName, prefill: isRoot ? '' : `@${c.author.displayName} ` });
    // Ô soạn nằm cuối luồng ⇒ mở hết phản hồi để người đọc thấy chỗ phản hồi sẽ xuất hiện.
    setExpanded(true);
  };

  const sendReply = async (body: string) => {
    if (!target) return;
    const created = await api.comments.create(slug, { body, parentId: target.parentId });
    bumpArticleComments(qc, slug, 1);
    if (!appendReply(qc, slug, { ...created, parentId: created.parentId ?? thread.id })) {
      void qc.invalidateQueries({ queryKey: commentsKey(slug) });
    }
    setTarget(null);
  };

  return (
    <li className="py-6 first:pt-2">
      <CommentItem slug={slug} comment={thread} onReply={startReply} onRequireLogin={onRequireLogin} />
      {(replies.length > 0 || target) && (
        <div
          role="group"
          aria-label={t('comments.repliesOf', { name: thread.author.displayName })}
          className="mt-4 ml-5 space-y-4 border-l-2 border-primary pl-4 sm:ml-[3.25rem] sm:pl-5"
        >
          {shown.map((r) => (
            <CommentItem key={r.id} slug={slug} comment={r} isReply onReply={startReply} onRequireLogin={onRequireLogin} />
          ))}
          {collapsible && (
            <button
              type="button"
              onClick={() => setExpanded((v) => !v)}
              aria-expanded={expanded}
              className="text-sm font-semibold text-primary-ink hover:underline"
            >
              {expanded
                ? t('comments.hideReplies', { count: hiddenCount, n: hiddenCount })
                : t('comments.showReplies', { count: hiddenCount, n: hiddenCount })}
            </button>
          )}
          {target && user && (
            <div>
              <p className="mb-2 text-xs font-semibold text-muted">{t('comments.replyingTo', { name: target.name })}</p>
              <CommentComposer
                key={target.parentId}
                user={user}
                initialValue={target.prefill}
                autoFocus
                placeholder={t('comments.replyPlaceholder')}
                submitLabel={t('comments.submitReply')}
                label={t('comments.replyTo', { name: target.name })}
                onSubmit={sendReply}
                onCancel={() => setTarget(null)}
              />
            </div>
          )}
        </div>
      )}
    </li>
  );
}

function CommentSkeleton() {
  return (
    <div className="flex gap-3 py-4">
      <Skeleton className="h-10 w-10 rounded-full" />
      <div className="flex-1 space-y-2">
        <Skeleton className="h-4 w-40" />
        <Skeleton className="h-4 w-full" />
        <Skeleton className="h-4 w-2/3" />
      </div>
    </div>
  );
}

/** Khối bình luận 2 cấp (A5 + A8). */
export function CommentsSection({ slug }: { slug: string }) {
  const { t } = useTranslation();
  const locale = useLocale();
  const qc = useQueryClient();
  const { user, loading: authLoading } = useAuth();
  const pathname = usePathname();
  const router = useRouter();
  const [sort, setSort] = useState<CommentSort>('newest');
  const next = `${pathname}#comments`;

  const query = useInfiniteQuery({
    queryKey: [...commentsKey(slug), sort],
    queryFn: ({ pageParam }) => api.comments.list(slug, { sort, page: pageParam, pageSize: PAGE_SIZE }),
    initialPageParam: 1,
    getNextPageParam: (last, all) => {
      const loaded = all.reduce((n, p) => n + p.items.length, 0);
      return loaded < last.total && last.items.length > 0 ? last.page + 1 : undefined;
    },
  });

  // Gộp các trang, bỏ trùng (bình luận mới chen vào có thể đẩy một luồng sang trang sau).
  const threads = useMemo(() => {
    const seen = new Set<string>();
    const out: CommentThread[] = [];
    for (const p of query.data?.pages ?? []) {
      for (const th of p.items) {
        if (seen.has(th.id)) continue;
        seen.add(th.id);
        out.push(th);
      }
    }
    return out;
  }, [query.data]);

  const pages = query.data?.pages;
  const count = pages && pages.length > 0 ? pages[pages.length - 1]!.commentCount : undefined;
  const requireLogin = () => router.push(loginHref(next));

  const createRoot = async (body: string) => {
    await api.comments.create(slug, { body });
    bumpArticleComments(qc, slug, 1);
    if (sort !== 'newest') setSort('newest');
    await qc.invalidateQueries({ queryKey: commentsKey(slug) });
  };

  const pill = (active: boolean) =>
    `rounded-full px-3.5 py-1.5 text-sm font-semibold transition-colors ${active ? 'bg-primary text-white' : 'text-muted hover:bg-surface-2 hover:text-ink'}`;
  const sortLabel: Record<CommentSort, string> = { newest: t('comments.newest'), oldest: t('comments.oldest'), top: t('comments.top') };

  return (
    <section id="comments" aria-labelledby="comments-title" className="scroll-mt-24">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <h2 id="comments-title" className="text-xl font-bold text-ink">
          {t('comments.title', { n: count === undefined ? '…' : formatNumber(count, locale) })}
        </h2>
        <div role="group" aria-label={t('comments.sortLabel')} className="inline-flex rounded-full border border-line bg-surface p-1">
          {SORTS.map((s) => (
            <button key={s} type="button" aria-pressed={sort === s} onClick={() => setSort(s)} className={pill(sort === s)}>
              {sortLabel[s]}
            </button>
          ))}
        </div>
      </div>

      <div className="mt-5">
        {authLoading ? (
          <Skeleton className="h-24 w-full rounded-card" />
        ) : user ? (
          <CommentComposer user={user} placeholder={t('comments.placeholder')} submitLabel={t('comments.submit')} onSubmit={createRoot} />
        ) : (
          <div className="flex flex-col items-start gap-3 rounded-card border border-dashed border-line bg-surface p-5 sm:flex-row sm:items-center sm:justify-between">
            <p className="flex items-center gap-2 text-sm text-ink">
              <MessageSquare aria-hidden="true" className="h-5 w-5 text-primary" />
              {t('comments.loginPrompt')}
            </p>
            <div className="flex gap-2">
              <Link href={loginHref(next)} rel="nofollow" className={btnPrimary}>
                {t('comments.loginCta')}
              </Link>
              <Link href={registerHref(next)} rel="nofollow" className={btnSecondary}>
                {t('comments.registerCta')}
              </Link>
            </div>
          </div>
        )}
      </div>

      <div className="mt-4">
        {query.isPending ? (
          <LoadingRegion>
            {[0, 1, 2].map((i) => (
              <CommentSkeleton key={i} />
            ))}
          </LoadingRegion>
        ) : query.isError && threads.length === 0 ? (
          <ErrorState onRetry={() => void query.refetch()} />
        ) : threads.length === 0 ? (
          <p className="py-8 text-center text-sm text-muted">{t('comments.empty')}</p>
        ) : (
          <>
            <ul className="divide-y divide-line">
              {threads.map((th) => (
                <Thread key={th.id} slug={slug} thread={th} onRequireLogin={requireLogin} />
              ))}
            </ul>
            {query.isFetchNextPageError && <ErrorState compact onRetry={() => void query.fetchNextPage()} />}
            {query.hasNextPage && (
              <div className="mt-2 flex justify-center">
                <button type="button" onClick={() => void query.fetchNextPage()} disabled={query.isFetchingNextPage} className={btnSecondary}>
                  {query.isFetchingNextPage ? t('comments.loadingMore') : t('comments.loadMore')}
                </button>
              </div>
            )}
          </>
        )}
      </div>

      <p className="mt-6 border-t border-line pt-4 text-xs text-muted">{t('comments.note')}</p>
    </section>
  );
}
