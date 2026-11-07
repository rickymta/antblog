'use client';

import { useCallback, useId, useRef, useState } from 'react';
import Link from 'next/link';
import { usePathname, useSearchParams } from 'next/navigation';
import { useTranslation } from 'react-i18next';
import { LogIn, LogOut, UserRound } from 'lucide-react';
import { Avatar } from '@tkh/ui';
import { useAuth } from '../../lib/auth';
import { useDismiss } from '../../hooks/useDismiss';
import { loginHref } from '../../lib/navigation';
import { btnPrimary, iconBtn } from '../../lib/ui';

/** Khung chờ (HTML máy chủ + lúc chưa biết phiên) — cũng là fallback của <Suspense> quanh UserMenu ở thanh đầu. */
export function UserMenuSkeleton() {
  return <span className="h-9 w-9 animate-pulse rounded-full bg-surface-2" aria-hidden="true" />;
}

/**
 * Avatar + menu tài khoản khi đã đăng nhập; nút "Đăng nhập" nền primary khi chưa.
 * Dùng useSearchParams ⇒ nơi đặt PHẢI bọc <Suspense> (trang bài/lộ trình là ISR — thiếu Suspense thì render lỗi).
 */
export function UserMenu({ inline = false, onNavigate }: { inline?: boolean; onNavigate?: () => void } = {}) {
  const { t } = useTranslation();
  const { user, loading, logout } = useAuth();
  const pathname = usePathname();
  const search = useSearchParams().toString();
  const [open, setOpen] = useState(false);
  const wrapRef = useRef<HTMLDivElement>(null);
  const btnRef = useRef<HTMLButtonElement>(null);
  const menuId = useId();
  const close = useCallback(() => setOpen(false), []);
  useDismiss(wrapRef, open, close, btnRef);

  if (loading) return <UserMenuSkeleton />;

  if (!user) {
    const href = loginHref(search ? `${pathname}?${search}` : pathname);
    if (inline) return <Link href={href} rel="nofollow" onClick={onNavigate} className={`${btnPrimary} w-full`}>{t('header.login')}</Link>;
    return (
      <>
        <Link href={href} rel="nofollow" className={`${btnPrimary} max-sm:hidden`}>
          {t('header.login')}
        </Link>
        <Link href={href} rel="nofollow" className={`${iconBtn} sm:hidden`} aria-label={t('header.login')}>
          <LogIn aria-hidden="true" className="h-[18px] w-[18px]" />
        </Link>
      </>
    );
  }

  if (inline) return (
    <div className="flex flex-wrap items-center gap-2">
      <Link href="/me" onClick={onNavigate} className="flex min-h-11 min-w-0 flex-1 items-center gap-2 rounded-ui px-3 text-sm font-semibold text-ink hover:bg-surface-2">
        <UserRound aria-hidden="true" className="h-5 w-5 shrink-0" />
        {t('header.profile')}
      </Link>
      <button type="button" onClick={() => { onNavigate?.(); void logout(); }} className="flex min-h-11 items-center gap-2 rounded-ui px-3 text-sm text-muted hover:bg-surface-2">
        <LogOut aria-hidden="true" className="h-5 w-5" />{t('header.logout')}
      </button>
    </div>
  );

  return (
    <div ref={wrapRef} className="relative">
      <button
        ref={btnRef}
        type="button"
        aria-expanded={open}
        aria-controls={menuId}
        aria-label={t('header.userMenu', { name: user.displayName })}
        onClick={() => setOpen((v) => !v)}
        className="inline-flex rounded-full"
      >
        <Avatar name={user.displayName} src={user.avatarUrl} size={36} />
      </button>
      {open && (
        <div id={menuId} className="absolute right-0 top-full z-50 mt-2 w-64 rounded-card border border-line bg-surface p-2 shadow-xl shadow-slate-900/10">
          <div className="flex items-center gap-3 border-b border-line px-2 pt-1 pb-3">
            <Avatar name={user.displayName} src={user.avatarUrl} size={40} />
            <div className="min-w-0">
              <p className="truncate text-sm font-semibold text-ink">{user.displayName}</p>
              <p className="truncate text-xs text-muted">{user.email}</p>
            </div>
          </div>
          <ul className="pt-1">
            <li>
              <Link href="/me" onClick={close} className="flex items-center gap-2 rounded-ui px-2 py-2 text-sm text-ink hover:bg-surface-2">
                <UserRound aria-hidden="true" className="h-4 w-4 text-muted" />
                {t('header.profile')}
              </Link>
            </li>
            <li>
              <button
                type="button"
                onClick={() => {
                  close();
                  void logout();
                }}
                className="flex w-full items-center gap-2 rounded-ui px-2 py-2 text-left text-sm text-ink hover:bg-surface-2"
              >
                <LogOut aria-hidden="true" className="h-4 w-4 text-muted" />
                {t('header.logout')}
              </button>
            </li>
          </ul>
        </div>
      )}
    </div>
  );
}
