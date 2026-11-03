import { useEffect, useState, type ReactNode } from 'react';
import { NavLink, Outlet, ScrollRestoration, useLocation } from 'react-router';
import { useTranslation } from 'react-i18next';
import {
  ExternalLink,
  FileText,
  FolderTree,
  LayoutDashboard,
  LogOut,
  Mail,
  Menu,
  MessageSquare,
  Route as RouteIcon,
  Users,
  X,
  type LucideIcon,
} from 'lucide-react';
import type { Role } from '@tkh/shared';
import { Button, IconButton, Spinner, ErrorState, cx } from '../components/ui';
import { RoleBadge } from '../components/domain';
import { useCurrentUser, useLogout, useSession } from '../lib/session';
import { errorMessage } from '../lib/errors';
import { SITE_URL } from '../lib/site';
import { LoginPage, NoAccessPage } from './LoginPage';
import { Logo } from './Logo';
import { PreferenceToggles } from './PreferenceToggles';

interface NavItem {
  to: string;
  labelKey:
    | 'nav.dashboard'
    | 'nav.articles'
    | 'nav.categories'
    | 'nav.learningPaths'
    | 'nav.comments'
    | 'nav.users'
    | 'nav.subscribers';
  icon: LucideIcon;
  roles: Role[];
  end?: boolean;
}

const NAV: NavItem[] = [
  { to: '/', labelKey: 'nav.dashboard', icon: LayoutDashboard, roles: ['admin', 'author'], end: true },
  { to: '/articles', labelKey: 'nav.articles', icon: FileText, roles: ['admin', 'author'] },
  { to: '/categories', labelKey: 'nav.categories', icon: FolderTree, roles: ['admin'] },
  { to: '/learning-paths', labelKey: 'nav.learningPaths', icon: RouteIcon, roles: ['admin'] },
  { to: '/comments', labelKey: 'nav.comments', icon: MessageSquare, roles: ['admin'] },
  { to: '/users', labelKey: 'nav.users', icon: Users, roles: ['admin'] },
  { to: '/subscribers', labelKey: 'nav.subscribers', icon: Mail, roles: ['admin'] },
];

/** Cổng phiên: chưa đăng nhập ⇒ màn đăng nhập (giữ nguyên URL); người đọc ⇒ báo không có quyền. */
export function SessionGate() {
  const { t, i18n } = useTranslation();
  const session = useSession();
  if (session.isPending) {
    return (
      <div className="flex min-h-dvh items-center justify-center">
        <Spinner label={t('auth.checking')} />
      </div>
    );
  }
  if (session.isError) {
    return (
      <div className="flex min-h-dvh items-center justify-center p-4">
        <div className="w-full max-w-lg rounded-card border border-line bg-surface">
          <ErrorState
            title={t('auth.sessionError')}
            message={errorMessage(session.error, t, i18n.language)}
            onRetry={() => void session.refetch()}
          />
        </div>
      </div>
    );
  }
  const user = session.data;
  if (!user) return <LoginPage />;
  if (user.role !== 'admin' && user.role !== 'author') return <NoAccessPage user={user} />;
  return <AppShell />;
}

function SidebarNav({ onNavigate }: { onNavigate?: () => void }) {
  const { t } = useTranslation();
  const user = useCurrentUser();
  return (
    <nav aria-label={t('nav.mainMenu')} className="flex flex-1 flex-col gap-1 p-3">
      {NAV.filter((n) => n.roles.includes(user.role)).map((n) => (
        <NavLink
          key={n.to}
          to={n.to}
          end={n.end}
          onClick={onNavigate}
          className={({ isActive }) =>
            cx(
              'flex items-center gap-3 rounded-ui px-3 py-2.5 text-sm font-medium transition-colors',
              isActive ? 'bg-primary text-white shadow-sm' : 'text-ink hover:bg-surface-2',
            )
          }
        >
          <n.icon className="size-4.5 shrink-0" aria-hidden />
          {t(n.labelKey)}
        </NavLink>
      ))}
      <div className="mt-auto border-t border-line pt-3">
        <a
          href={SITE_URL || '/'}
          target="_blank"
          rel="noreferrer"
          className="flex items-center gap-3 rounded-ui px-3 py-2.5 text-sm font-medium text-muted hover:bg-surface-2 hover:text-ink"
        >
          <ExternalLink className="size-4.5 shrink-0" aria-hidden />
          {t('nav.viewSite')}
        </a>
      </div>
    </nav>
  );
}

function AppShell() {
  const { t } = useTranslation();
  const user = useCurrentUser();
  const logout = useLogout();
  const [drawer, setDrawer] = useState(false);
  const [loggingOut, setLoggingOut] = useState(false);
  const location = useLocation();

  // Đổi trang ⇒ đóng ngăn kéo (điện thoại/tablet).
  useEffect(() => setDrawer(false), [location.pathname]);

  useEffect(() => {
    if (!drawer) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') setDrawer(false);
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [drawer]);

  return (
    <div className="min-h-dvh bg-page">
      <a
        href="#main"
        className="sr-only focus:not-sr-only focus:fixed focus:top-2 focus:left-2 focus:z-[70] focus:rounded-ui focus:bg-primary focus:px-3 focus:py-2 focus:text-white"
      >
        {t('app.skipToContent')}
      </a>

      {/* Thanh bên cố định từ lg */}
      <aside className="fixed inset-y-0 left-0 z-30 hidden w-64 flex-col border-r border-line bg-surface lg:flex">
        <div className="flex h-16 items-center border-b border-line px-4">
          <Logo />
        </div>
        <SidebarNav />
      </aside>

      {/* Ngăn kéo cho màn hẹp */}
      {drawer && (
        <div className="fixed inset-0 z-40 lg:hidden">
          <button
            type="button"
            className="absolute inset-0 bg-slate-950/55"
            aria-label={t('nav.closeMenu')}
            onClick={() => setDrawer(false)}
          />
          <aside id="admin-drawer" className="absolute inset-y-0 left-0 flex w-72 max-w-[85vw] flex-col border-r border-line bg-surface shadow-2xl">
            <div className="flex h-16 items-center justify-between border-b border-line px-4">
              <Logo />
              <IconButton icon={X} label={t('nav.closeMenu')} onClick={() => setDrawer(false)} autoFocus />
            </div>
            <SidebarNav onNavigate={() => setDrawer(false)} />
          </aside>
        </div>
      )}

      <div className="lg:pl-64">
        <header className="sticky top-0 z-20 flex h-16 items-center gap-3 border-b border-line bg-surface/90 px-4 backdrop-blur sm:px-6">
          <IconButton
            icon={Menu}
            label={t('nav.openMenu')}
            className="lg:hidden"
            aria-expanded={drawer}
            aria-controls="admin-drawer"
            onClick={() => setDrawer(true)}
          />
          <span className="lg:hidden">
            <Logo compact />
          </span>
          <div className="ml-auto flex items-center gap-2 sm:gap-3">
            <div className="hidden min-w-0 items-center gap-2 text-right sm:flex">
              <span className="max-w-[12rem] truncate text-sm font-semibold text-ink">{user.displayName}</span>
              <RoleBadge role={user.role} />
            </div>
            <PreferenceToggles />
            <Button
              size="sm"
              icon={LogOut}
              loading={loggingOut}
              onClick={async () => {
                setLoggingOut(true);
                await logout();
              }}
            >
              <span className="hidden sm:inline">{t('topbar.logout')}</span>
              <span className="sr-only sm:hidden">{t('topbar.logout')}</span>
            </Button>
          </div>
        </header>
        <main id="main" tabIndex={-1} className="mx-auto w-full max-w-7xl px-4 py-6 focus:outline-none sm:px-6 lg:py-8">
          <Outlet />
        </main>
        <ScrollRestoration />
      </div>
    </div>
  );
}

/** Mục chỉ dành cho admin: tác giả vào bằng URL thì thấy thông báo thay vì lỗi 403 của API. */
export function AdminOnly({ children }: { children: ReactNode }) {
  const { t } = useTranslation();
  const user = useCurrentUser();
  if (user.role === 'admin') return <>{children}</>;
  return (
    <div className="rounded-card border border-line bg-surface">
      <ErrorState title={t('common.noPermissionTitle')} message={t('common.noPermissionBody')} />
    </div>
  );
}
