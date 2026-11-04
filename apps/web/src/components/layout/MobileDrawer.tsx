'use client';

import { useEffect, useRef } from 'react';
import { createPortal } from 'react-dom';
import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { useTranslation } from 'react-i18next';
import { X } from 'lucide-react';
import type { Category } from '@tkh/shared';
import { iconBtn } from '../../lib/ui';
import { LogoMark } from './Logo';
import { isActivePath, MAIN_LINKS } from './navLinks';
import { I18N_ENABLED } from '../../i18n/config';
import { LanguageToggle } from './Toggles';
import { TopicLinks } from './TopicsMenu';

/** Ngăn kéo trái trên điện thoại: menu chính + danh sách chủ đề + VI/EN. Escape / bấm nền mờ để đóng; khoá cuộn trang khi mở. */
export function MobileDrawer({ open, onClose, categories }: { open: boolean; onClose: () => void; categories: Category[] | null }) {
  const { t } = useTranslation();
  const pathname = usePathname();
  const panelRef = useRef<HTMLDivElement>(null);
  const closeRef = useRef<HTMLButtonElement>(null);

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
            {MAIN_LINKS.map((l) => {
              const active = isActivePath(pathname, l.to, l.end);
              return (
                <li key={l.to}>
                  <Link
                    href={l.to}
                    onClick={onClose}
                    aria-current={active ? 'page' : undefined}
                    className={`block rounded-ui px-3 py-2.5 text-sm font-semibold ${active ? 'bg-primary-soft text-primary-ink' : 'text-ink hover:bg-surface-2'}`}
                  >
                    {t(l.label)}
                  </Link>
                </li>
              );
            })}
          </ul>
        </nav>
        <div className="border-t border-line px-3 py-3">
          <p className="px-2 pb-2 text-xs font-bold tracking-wide text-muted uppercase">{t('nav.topics')}</p>
          {categories ? (
            <TopicLinks categories={categories} onNavigate={onClose} dense />
          ) : (
            <p className="px-2 text-sm text-muted">{t('nav.topicsLoadFailed')}</p>
          )}
        </div>
        {I18N_ENABLED && (
          <div className="mt-auto flex items-center justify-between border-t border-line px-5 py-4">
            <span className="text-sm text-muted">{t('header.language')}</span>
            <LanguageToggle />
          </div>
        )}
      </div>
    </div>,
    document.body,
  );
}
