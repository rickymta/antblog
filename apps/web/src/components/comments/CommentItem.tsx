'use client';

import { useCallback, useId, useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useQueryClient } from '@tanstack/react-query';
import { CornerDownRight, Ellipsis, Pencil, ThumbsUp, Trash } from 'lucide-react';
import { Avatar } from '@tkh/ui';
import { api, formatCompact, formatRelative, type Comment } from '@tkh/shared';
import { formatDay } from '../../lib/dates';
import { useLocale } from '../../i18n';
import { useAuth } from '../../lib/auth';
import { isUnauthenticated } from '../../lib/errors';
import { useDismiss } from '../../hooks/useDismiss';
import { CommentComposer } from './CommentComposer';
import { bumpArticleComments, commentsKey, patchComment } from './commentCache';

const actionBtn = 'inline-flex items-center gap-1.5 rounded-ui px-2 py-1 text-xs font-semibold transition-colors';

/**
 * Một bình luận (A8): avatar, tên đậm · thời gian tương đối, nội dung VĂN BẢN THUẦN (giữ xuống dòng, không bao giờ render HTML/Markdown),
 * 👍 số (bật/tắt) · Trả lời; chủ bình luận có Sửa/Xoá trong menu ⋯. Đã xoá ⇒ "Bình luận đã bị xoá".
 */
export function CommentItem({
  slug,
  comment,
  isReply = false,
  onReply,
  onRequireLogin,
}: {
  slug: string;
  comment: Comment;
  isReply?: boolean;
  onReply: (target: Comment) => void;
  onRequireLogin: () => void;
}) {
  const { t } = useTranslation();
  const locale = useLocale();
  const qc = useQueryClient();
  const { user, setUser } = useAuth();
  const [editing, setEditing] = useState(false);
  const [menuOpen, setMenuOpen] = useState(false);
  const [confirmDelete, setConfirmDelete] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const menuRef = useRef<HTMLDivElement>(null);
  const menuBtnRef = useRef<HTMLButtonElement>(null);
  const menuId = useId();
  const closeMenu = useCallback(() => setMenuOpen(false), []);
  useDismiss(menuRef, menuOpen, closeMenu, menuBtnRef);

  const deleted = comment.status === 'deleted';
  const isOwner = Boolean(user && user.id === comment.author.id);
  const size = isReply ? 32 : 40;

  const fail = (e: unknown) => {
    if (isUnauthenticated(e)) {
      setUser(null);
      onRequireLogin();
    } else setError(t('comments.actionError'));
  };

  const toggleLike = async () => {
    if (!user) return onRequireLogin();
    if (busy) return;
    const was = comment.likedByMe;
    setBusy(true);
    setError(null);
    patchComment(qc, slug, comment.id, (c) => ({ ...c, likedByMe: !was, likeCount: Math.max(0, c.likeCount + (was ? -1 : 1)) }));
    try {
      const res = was ? await api.comments.unlike(comment.id) : await api.comments.like(comment.id);
      patchComment(qc, slug, comment.id, (c) => ({ ...c, likeCount: res.likeCount, likedByMe: res.likedByMe }));
    } catch (e) {
      patchComment(qc, slug, comment.id, (c) => ({ ...c, likedByMe: was, likeCount: Math.max(0, c.likeCount + (was ? 1 : -1)) }));
      fail(e);
    } finally {
      setBusy(false);
    }
  };

  const saveEdit = async (body: string) => {
    const updated = await api.comments.update(comment.id, body);
    patchComment(qc, slug, comment.id, (c) => ({ ...c, body: updated.body, editedAt: updated.editedAt }));
    setEditing(false);
  };

  const remove = async () => {
    setBusy(true);
    setError(null);
    try {
      await api.comments.remove(comment.id);
      patchComment(qc, slug, comment.id, (c) => ({ ...c, status: 'deleted', body: '' }));
      bumpArticleComments(qc, slug, -1);
      setConfirmDelete(false);
      // Số bình luận và thứ tự "Nổi bật" do máy chủ tính lại.
      void qc.invalidateQueries({ queryKey: commentsKey(slug) });
    } catch (e) {
      fail(e);
    } finally {
      setBusy(false);
    }
  };

  const time = (
    <time dateTime={comment.createdAt} title={formatDay(comment.createdAt, locale)} className="text-xs text-muted">
      {formatRelative(comment.createdAt, locale)}
    </time>
  );

  return (
    <article className="flex gap-3" aria-label={comment.author.displayName}>
      <Avatar name={comment.author.displayName} src={comment.author.avatarUrl} size={size} className={deleted ? 'opacity-50' : ''} />
      <div className="min-w-0 flex-1">
        <div className="flex flex-wrap items-center gap-x-2 gap-y-0.5">
          <span className={`text-sm font-bold ${deleted ? 'text-muted' : 'text-ink'}`}>{comment.author.displayName}</span>
          <span aria-hidden="true" className="text-muted">
            ·
          </span>
          {time}
          {comment.editedAt && !deleted && <span className="text-xs text-muted italic">({t('comments.edited')})</span>}
        </div>

        {comment.replyTo && !deleted && (
          <p className="mt-0.5 inline-flex items-center gap-1 text-xs font-semibold text-primary-ink">
            <CornerDownRight aria-hidden="true" className="h-3.5 w-3.5" />
            <span className="sr-only">{t('comments.replyTo', { name: comment.replyTo.displayName })}</span>
            <span aria-hidden="true">@{comment.replyTo.displayName}</span>
          </p>
        )}

        {deleted ? (
          <p className="mt-1 text-sm text-muted italic">{t('comments.deleted')}</p>
        ) : editing && user ? (
          <div className="mt-2">
            <CommentComposer
              user={user}
              showAvatar={false}
              initialValue={comment.body}
              autoFocus
              placeholder={t('comments.placeholder')}
              submitLabel={t('comments.save')}
              onSubmit={saveEdit}
              onCancel={() => setEditing(false)}
            />
          </div>
        ) : (
          <p className="mt-1 text-sm leading-relaxed break-words whitespace-pre-wrap text-ink">{comment.body}</p>
        )}

        {!deleted && !editing && (
          <div className="mt-1.5 -ml-2 flex flex-wrap items-center gap-1">
            <button
              type="button"
              onClick={() => void toggleLike()}
              aria-pressed={comment.likedByMe}
              className={`${actionBtn} ${comment.likedByMe ? 'text-primary-ink' : 'text-muted hover:bg-surface-2 hover:text-ink'}`}
            >
              <ThumbsUp aria-hidden="true" className="h-4 w-4" fill={comment.likedByMe ? 'currentColor' : 'none'} />
              {/* Tên truy cập chứa đúng số nhìn thấy (WCAG 2.5.3): hành động cho trình đọc màn hình + số đếm hiển thị. */}
              <span className="sr-only">{comment.likedByMe ? t('comments.unlike') : t('comments.like')}: </span>
              <span>{formatCompact(comment.likeCount, locale)}</span>
            </button>
            <button
              type="button"
              onClick={() => (user ? onReply(comment) : onRequireLogin())}
              className={`${actionBtn} text-muted hover:bg-surface-2 hover:text-ink`}
            >
              {t('comments.reply')}
            </button>
            {isOwner && (
              <div ref={menuRef} className="relative">
                <button
                  ref={menuBtnRef}
                  type="button"
                  aria-label={t('comments.moreActions')}
                  aria-expanded={menuOpen}
                  aria-controls={menuId}
                  onClick={() => setMenuOpen((v) => !v)}
                  className={`${actionBtn} text-muted hover:bg-surface-2 hover:text-ink`}
                >
                  <Ellipsis aria-hidden="true" className="h-4 w-4" />
                </button>
                {menuOpen && (
                  <ul id={menuId} className="absolute left-0 top-full z-20 mt-1 w-36 rounded-ui border border-line bg-surface p-1 shadow-lg">
                    <li>
                      <button
                        type="button"
                        onClick={() => {
                          closeMenu();
                          setConfirmDelete(false);
                          setEditing(true);
                        }}
                        className="flex w-full items-center gap-2 rounded-[6px] px-2 py-1.5 text-left text-sm text-ink hover:bg-surface-2"
                      >
                        <Pencil aria-hidden="true" className="h-3.5 w-3.5 text-muted" />
                        {t('comments.edit')}
                      </button>
                    </li>
                    <li>
                      <button
                        type="button"
                        onClick={() => {
                          closeMenu();
                          setConfirmDelete(true);
                        }}
                        className="flex w-full items-center gap-2 rounded-[6px] px-2 py-1.5 text-left text-sm text-red-700 hover:bg-surface-2 dark:text-red-400"
                      >
                        <Trash aria-hidden="true" className="h-3.5 w-3.5" />
                        {t('comments.delete')}
                      </button>
                    </li>
                  </ul>
                )}
              </div>
            )}
          </div>
        )}

        {confirmDelete && !deleted && (
          <div role="alertdialog" aria-label={t('comments.confirmDelete')} className="mt-2 flex flex-wrap items-center gap-2 rounded-ui border border-line bg-surface-2 px-3 py-2 text-sm">
            <span className="font-medium text-ink">{t('comments.confirmDelete')}</span>
            <button
              type="button"
              disabled={busy}
              onClick={() => void remove()}
              className="rounded-ui bg-red-600 px-3 py-1 text-xs font-semibold text-white hover:bg-red-700 disabled:opacity-50"
            >
              {t('comments.delete')}
            </button>
            <button type="button" onClick={() => setConfirmDelete(false)} className="rounded-ui px-3 py-1 text-xs font-semibold text-ink hover:bg-surface">
              {t('common.cancel')}
            </button>
          </div>
        )}
        {error && (
          <p role="alert" className="mt-1 text-xs font-medium text-red-700 dark:text-red-400">
            {error}
          </p>
        )}
      </div>
    </article>
  );
}
