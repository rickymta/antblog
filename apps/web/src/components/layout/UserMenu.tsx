import { useCallback, useId, useRef, useState } from 'react';
import { Link, useLocation } from 'react-router';
import { useTranslation } from 'react-i18next';
import { LogIn, LogOut, UserRound } from 'lucide-react';
import { Avatar } from '@tkh/ui';
import { useAuth } from '../../lib/auth';
import { useDismiss } from '../../hooks/useDismiss';
import { loginHref } from '../../lib/navigation';
import { btnPrimary, iconBtn } from '../../lib/ui';

/** Avatar + menu tài khoản khi đã đăng nhập; nút "Đăng nhập" nền primary khi chưa. */
export function UserMenu() {
  const { t } = useTranslation();
  const { user, loading, logout } = useAuth();
  const location = useLocation();
  const [open, setOpen] = useState(false);
  const wrapRef = useRef<HTMLDivElement>(null);
  const btnRef = useRef<HTMLButtonElement>(null);
  const menuId = useId();
  const close = useCallback(() => setOpen(false), []);
  useDismiss(wrapRef, open, close, btnRef);

  if (loading) return <span className="h-9 w-9 animate-pulse rounded-full bg-surface-2" aria-hidden="true" />;

  if (!user) {
    const href = loginHref(location.pathname + location.search);
    return (
      <>
        <Link to={href} className={`${btnPrimary} max-sm:hidden`}>
          {t('header.login')}
        </Link>
        <Link to={href} className={`${iconBtn} sm:hidden`} aria-label={t('header.login')}>
          <LogIn aria-hidden="true" className="h-[18px] w-[18px]" />
        </Link>
      </>
    );
  }

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
              <Link to="/me" onClick={close} className="flex items-center gap-2 rounded-ui px-2 py-2 text-sm text-ink hover:bg-surface-2">
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
