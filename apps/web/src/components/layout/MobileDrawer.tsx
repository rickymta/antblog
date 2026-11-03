import { useEffect, useRef } from 'react';
import { createPortal } from 'react-dom';
import { NavLink } from 'react-router';
import { useTranslation } from 'react-i18next';
import { useQuery } from '@tanstack/react-query';
import { X } from 'lucide-react';
import { Skeleton } from '@tkh/ui';
import { iconBtn } from '../../lib/ui';
import { LogoMark } from './Logo';
import { MAIN_LINKS } from './navLinks';
import { LanguageToggle } from './Toggles';
import { categoriesQuery, TopicLinks } from './TopicsMenu';

/** Ngăn kéo trái trên điện thoại: menu chính + danh sách chủ đề + VI/EN. Escape / bấm nền mờ để đóng; khoá cuộn trang khi mở. */
export function MobileDrawer({ open, onClose }: { open: boolean; onClose: () => void }) {
  const { t } = useTranslation();
  const panelRef = useRef<HTMLDivElement>(null);
  const closeRef = useRef<HTMLButtonElement>(null);
  const categories = useQuery({ ...categoriesQuery, enabled: open });

  useEffect(() => {
    if (!open) return;
    const prevOverflow = document.body.style.overflow;
    document.body.style.overflow = 'hidden';
    closeRef.current?.focus();
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose();
      // Giữ focus trong ngăn kéo khi Tab.
      if (e.key === 'Tab' && panelRef.current) {
        const focusables = panelRef.current.querySelectorAll<HTMLElement>('a[href], button:not([disabled])');
        const first = focusables[0];
        const last = focusables[focusables.length - 1];
        if (!first || !last) return;
        if (e.shiftKey && document.activeElement === first) {
          e.preventDefault();
          last.focus();
        } else if (!e.shiftKey && document.activeElement === last) {
          e.preventDefault();
          first.focus();
        }
      }
    };
    document.addEventListener('keydown', onKey);
    return () => {
      document.body.style.overflow = prevOverflow;
      document.removeEventListener('keydown', onKey);
    };
  }, [open, onClose]);

  if (!open) return null;

  // Portal ra <body>: thanh đầu có backdrop-filter nên `fixed` bên trong nó bị giới hạn trong thanh đầu.
  return createPortal(
    <div className="fixed inset-0 z-50 lg:hidden">
      <div className="absolute inset-0 bg-slate-950/50" onClick={onClose} aria-hidden="true" />
      <div
        ref={panelRef}
        role="dialog"
        aria-modal="true"
        aria-label={t('nav.mobileNav')}
        className="absolute inset-y-0 left-0 flex w-[min(20rem,88vw)] flex-col overflow-y-auto border-r border-line bg-surface shadow-2xl"
      >
        <div className="flex items-center justify-between border-b border-line px-4 py-3">
          <span className="flex items-center gap-2 font-bold text-ink">
            <LogoMark className="h-8 w-8 text-xs" />
            {t('meta.siteName')}
          </span>
          <button ref={closeRef} type="button" onClick={onClose} className={iconBtn} aria-label={t('nav.closeMenu')}>
            <X aria-hidden="true" className="h-5 w-5" />
          </button>
        </div>
        <nav aria-label={t('nav.mainNav')} className="px-3 py-3">
          <ul className="space-y-1">
            {MAIN_LINKS.map((l) => (
              <li key={l.to}>
                <NavLink
                  to={l.to}
                  end={l.end}
                  onClick={onClose}
                  className={({ isActive }) =>
                    `block rounded-ui px-3 py-2.5 text-sm font-semibold ${isActive ? 'bg-primary-soft text-primary-ink' : 'text-ink hover:bg-surface-2'}`
                  }
                >
                  {t(l.label)}
                </NavLink>
              </li>
            ))}
          </ul>
        </nav>
        <div className="border-t border-line px-3 py-3">
          <p className="px-2 pb-2 text-xs font-bold tracking-wide text-muted uppercase">{t('nav.topics')}</p>
          {categories.isPending ? (
            <div className="space-y-2 px-2">
              {[0, 1, 2].map((i) => (
                <Skeleton key={i} className="h-8 w-full" />
              ))}
            </div>
          ) : categories.isError ? (
            <p className="px-2 text-sm text-muted">{t('nav.topicsLoadFailed')}</p>
          ) : (
            <TopicLinks categories={categories.data} onNavigate={onClose} dense />
          )}
        </div>
        <div className="mt-auto flex items-center justify-between border-t border-line px-5 py-4">
          <span className="text-sm text-muted">{t('header.language')}</span>
          <LanguageToggle />
        </div>
      </div>
    </div>,
    document.body,
  );
}
