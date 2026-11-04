'use client';

import { useEffect, useId, useRef, useState, type FormEvent } from 'react';
import Link from 'next/link';
import { useRouter } from 'next/navigation';
import { useTranslation } from 'react-i18next';
import { useMutation } from '@tanstack/react-query';
import { BookmarkX, CircleCheck, LogOut } from 'lucide-react';
import { Avatar, Skeleton } from '@tkh/ui';
import { api, type User } from '@tkh/shared';
import { formatDay } from '../lib/dates';
import { useLocale } from '../i18n';
import { articleHref } from '../components/common/ArticleCard';
import { TextField } from '../components/common/FormField';
import { PageHeader } from '../components/common/PageHeader';
import { EmptyState, LoadingRegion } from '../components/common/States';
import { useAuth } from '../lib/auth';
import { apiMessage, fieldErrors } from '../lib/errors';
import { loginHref } from '../lib/navigation';
import { toggleBookmark, useBookmarks } from '../lib/reading';
import { btnPrimary, btnSecondary, card, container, inputClass } from '../lib/ui';

const URL_RE = /^https?:\/\/\S+$/i;

function ProfileForm({ user }: { user: User }) {
  const { t } = useTranslation();
  const locale = useLocale();
  const { setUser } = useAuth();
  const bioId = useId();
  const [displayName, setDisplayName] = useState(user.displayName);
  const [avatarUrl, setAvatarUrl] = useState(user.avatarUrl ?? '');
  const [bio, setBio] = useState(user.bio ?? '');
  const [local, setLocal] = useState<Record<string, string>>({});
  const save = useMutation({ mutationFn: api.auth.updateMe, onSuccess: (res) => setUser(res.user) });

  const submit = (e: FormEvent) => {
    e.preventDefault();
    const errs: Record<string, string> = {};
    const name = displayName.trim();
    const url = avatarUrl.trim();
    if (name.length < 2 || name.length > 60) errs.displayName = t('auth.displayNameLength');
    if (url && (!URL_RE.test(url) || url.length > 500)) errs.avatarUrl = t('profile.invalidUrl');
    if (bio.length > 300) errs.bio = t('profile.bioHint');
    setLocal(errs);
    if (Object.keys(errs).length > 0) return;
    // Chỉ gửi trường đã đổi; chuỗi rỗng = xoá ảnh đại diện / giới thiệu đang có.
    const input: { displayName?: string; avatarUrl?: string; bio?: string } = {};
    if (name !== user.displayName) input.displayName = name;
    if (url !== (user.avatarUrl ?? '')) input.avatarUrl = url;
    if (bio.trim() !== (user.bio ?? '')) input.bio = bio.trim();
    save.mutate(input);
  };

  const server = fieldErrors(save.error);
  const err = (f: string) => local[f] ?? server[f];
  const previewUrl = URL_RE.test(avatarUrl.trim()) ? avatarUrl.trim() : undefined;

  return (
    <form onSubmit={submit} noValidate className={`space-y-5 p-5 sm:p-6 ${card}`}>
      <div className="flex items-center gap-4">
        <Avatar name={displayName.trim() || user.displayName} src={previewUrl} size={64} label={t('profile.preview')} />
        <div className="min-w-0">
          <p className="truncate font-semibold text-ink">{displayName.trim() || user.displayName}</p>
          <p className="truncate text-sm text-muted">{user.email}</p>
          <p className="text-xs text-muted">{t('profile.memberSince', { date: formatDay(user.createdAt, locale) })}</p>
        </div>
      </div>
      <TextField label={t('profile.displayName')} value={displayName} onChange={(e) => setDisplayName(e.target.value)} maxLength={60} error={err('displayName')} autoComplete="nickname" />
      <TextField
        label={t('profile.avatarUrl')}
        type="url"
        inputMode="url"
        value={avatarUrl}
        onChange={(e) => setAvatarUrl(e.target.value)}
        hint={t('profile.avatarUrlHint')}
        error={err('avatarUrl')}
        maxLength={500}
        placeholder="https://"
      />
      <div>
        <label htmlFor={bioId} className="mb-1.5 block text-sm font-semibold text-ink">
          {t('profile.bio')}
        </label>
        <textarea
          id={bioId}
          value={bio}
          onChange={(e) => setBio(e.target.value)}
          rows={4}
          maxLength={300}
          aria-invalid={Boolean(err('bio'))}
          aria-describedby={`${bioId}-hint`}
          className={`${inputClass} resize-y`}
        />
        <p id={`${bioId}-hint`} className={`mt-1 text-xs ${err('bio') ? 'font-medium text-red-700 dark:text-red-400' : 'text-muted'}`}>
          {err('bio') ?? t('profile.bioHint')} ({bio.length}/300)
        </p>
      </div>
      <div className="flex flex-wrap items-center gap-3">
        <button type="submit" disabled={save.isPending} className={btnPrimary}>
          {save.isPending ? t('profile.saving') : t('profile.save')}
        </button>
        <div aria-live="polite" className="text-sm">
          {save.isSuccess && (
            <span className="inline-flex items-center gap-1.5 font-medium text-emerald-700 dark:text-emerald-400">
              <CircleCheck aria-hidden="true" className="h-4 w-4" />
              {t('profile.saved')}
            </span>
          )}
          {save.isError && Object.keys(server).length === 0 && (
            <span role="alert" className="font-medium text-red-700 dark:text-red-400">
              {apiMessage(save.error) ?? t('profile.error')}
            </span>
          )}
        </div>
      </div>
    </form>
  );
}

function SavedArticles() {
  const { t } = useTranslation();
  const bookmarks = useBookmarks();
  return (
    <section aria-labelledby="saved-title" className={`p-5 sm:p-6 ${card}`}>
      <h2 id="saved-title" className="text-lg font-bold text-ink">
        {t('profile.bookmarks')}
      </h2>
      <p className="mt-1 text-sm text-muted">{t('profile.bookmarksHint')}</p>
      <div className="mt-4">
        {bookmarks.length === 0 ? (
          <EmptyState title={t('profile.bookmarksEmpty')} />
        ) : (
          <ul className="divide-y divide-line">
            {bookmarks.map((b) => (
              <li key={b.slug} className="flex items-center gap-3 py-3">
                <div className="min-w-0 flex-1">
                  <Link href={articleHref(b.slug)} className="line-clamp-2 font-semibold text-ink hover:text-primary-ink hover:underline">
                    {b.title}
                  </Link>
                  <p className="text-xs text-muted">{t('common.minRead', { count: b.readingMinutes })}</p>
                </div>
                <button
                  type="button"
                  onClick={() => toggleBookmark(b)}
                  aria-label={t('profile.removeBookmark', { title: b.title })}
                  className="inline-flex h-9 w-9 shrink-0 items-center justify-center rounded-ui text-muted hover:bg-surface-2 hover:text-ink"
                >
                  <BookmarkX aria-hidden="true" className="h-4 w-4" />
                </button>
              </li>
            ))}
          </ul>
        )}
      </div>
    </section>
  );
}

export function ProfileView() {
  const { t } = useTranslation();
  const { user, loading, logout } = useAuth();
  const router = useRouter();
  const leaving = useRef(false);
  // Chưa đăng nhập ⇒ sang trang đăng nhập rồi quay lại đây (trừ khi vừa bấm Đăng xuất ở trang này).
  useEffect(() => {
    if (!loading && !user && !leaving.current) router.replace(loginHref('/me'));
  }, [loading, user, router]);

  if (loading || !user) {
    return (
      <LoadingRegion className={`${container} max-w-3xl space-y-4 py-10`}>
        <Skeleton className="h-10 w-1/3" />
        <Skeleton className="h-80 w-full rounded-card" />
      </LoadingRegion>
    );
  }

  return (
    <div className={`${container} max-w-3xl`}>
      <PageHeader title={t('profile.title')} subtitle={t('profile.subtitle')}>
        <button
          type="button"
          onClick={() => {
            // Rời trang trước, nếu không trang này sẽ chuyển sang /login khi phiên vừa mất.
            leaving.current = true;
            router.push('/');
            void logout();
          }}
          className={btnSecondary}
        >
          <LogOut aria-hidden="true" className="h-4 w-4" />
          {t('profile.logout')}
        </button>
      </PageHeader>
      <div className="space-y-6">
        {/* key theo id: đổi tài khoản thì form khởi tạo lại từ dữ liệu mới. */}
        <ProfileForm key={user.id} user={user} />
        <SavedArticles />
      </div>
    </div>
  );
}
