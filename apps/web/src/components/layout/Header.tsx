'use client';

import { useCallback, useEffect, useRef, useState } from 'react';
import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { useTranslation } from 'react-i18next';
import { Menu, Search, X } from 'lucide-react';
import type { Category } from '@tkh/shared';
import { container, iconBtn } from '../../lib/ui';
import { Logo } from './Logo';
import { MobileDrawer } from './MobileDrawer';
import { isActivePath, MAIN_LINKS } from './navLinks';
import { SearchBox } from './SearchBox';
import { LanguageToggle, ThemeToggle } from './Toggles';
import { TopicsMenu } from './TopicsMenu';
import { UserMenu } from './UserMenu';

const navItem = (active: boolean) =>
  `rounded-ui px-3 py-2 text-sm font-medium whitespace-nowrap transition-colors ${active ? 'text-primary-ink' : 'text-muted hover:bg-surface-2 hover:text-ink'}`;

/** Thanh đầu dính (docs/ui-guide.md §2). Màn hẹp: menu vào ☰ (< 1024px); dưới 1280px ô tìm kiếm thành biểu tượng mở một hàng tìm kiếm. */
export function Header({ categories }: { categories: Category[] | null }) {
  const { t } = useTranslation();
  const pathname = usePathname();
  const [drawerOpen, setDrawerOpen] = useState(false);
  const [searchOpen, setSearchOpen] = useState(false);
  const desktopSearch = useRef<HTMLInputElement>(null);
  const mobileSearch = useRef<HTMLInputElement>(null);
  const closeDrawer = useCallback(() => setDrawerOpen(false), []);

  // Đổi trang ⇒ đóng hàng tìm kiếm điện thoại.
  useEffect(() => setSearchOpen(false), [pathname]);

  useEffect(() => {
    if (searchOpen) mobileSearch.current?.focus();
  }, [searchOpen]);

  // Ctrl/Cmd + K: focus ô tìm kiếm đang hiển thị.
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'k') {
        e.preventDefault();
        const desktop = desktopSearch.current;
        if (desktop && desktop.offsetParent !== null) desktop.focus();
        else setSearchOpen(true);
      }
    };
    document.addEventListener('keydown', onKey);
    return () => document.removeEventListener('keydown', onKey);
  }, []);

  const navLink = (to: string, end: boolean, label: string) => {
    const active = isActivePath(pathname, to, end);
    return (
      <Link key={to} href={to} aria-current={active ? 'page' : undefined} className={navItem(active)}>
        {label}
      </Link>
    );
  };

  return (
    <header className="sticky top-0 z-40 border-b border-line bg-surface/90 backdrop-blur supports-[backdrop-filter]:bg-surface/80">
      <div className={`${container} flex h-16 items-center gap-2 sm:gap-3`}>
        <button type="button" className={`${iconBtn} lg:hidden`} onClick={() => setDrawerOpen(true)} aria-label={t('nav.openMenu')} aria-expanded={drawerOpen}>
          <Menu aria-hidden="true" className="h-5 w-5" />
        </button>
        <Logo />
        <nav aria-label={t('nav.mainNav')} className="ml-4 hidden items-center gap-0.5 lg:flex">
          {MAIN_LINKS.slice(0, 3).map((l) => navLink(l.to, l.end, t(l.label)))}
          <TopicsMenu categories={categories} />
          {navLink('/about', false, t('nav.about'))}
        </nav>
        <div className="ml-auto flex items-center gap-1.5 sm:gap-2">
          <SearchBox ref={desktopSearch} className="hidden w-56 xl:block 2xl:w-64" />
          <button
            type="button"
            className={`${iconBtn} xl:hidden`}
            onClick={() => setSearchOpen((v) => !v)}
            aria-label={searchOpen ? t('header.closeSearch') : t('header.openSearch')}
            aria-expanded={searchOpen}
          >
            {searchOpen ? <X aria-hidden="true" className="h-5 w-5" /> : <Search aria-hidden="true" className="h-5 w-5" />}
          </button>
          <LanguageToggle />
          <ThemeToggle />
          <UserMenu />
        </div>
      </div>
      {searchOpen && (
        <div className={`${container} pb-3 xl:hidden`}>
          <SearchBox ref={mobileSearch} showHint={false} onSubmitted={() => setSearchOpen(false)} />
        </div>
      )}
      <MobileDrawer open={drawerOpen} onClose={closeDrawer} categories={categories} />
    </header>
  );
}
