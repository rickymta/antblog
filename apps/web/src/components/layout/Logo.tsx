'use client';

import Link from 'next/link';
import { useTranslation } from 'react-i18next';

export function LogoMark({ className = 'h-9 w-9 text-sm' }: { className?: string }) {
  return (
    <span
      aria-hidden="true"
      className={`inline-flex shrink-0 items-center justify-center rounded-ui bg-gradient-to-br from-primary to-secondary font-mono font-bold text-white ${className}`}
    >
      {'</>'}
    </span>
  );
}

export function Logo({ onNavigate }: { onNavigate?: () => void }) {
  const { t } = useTranslation();
  return (
    <Link href="/" onClick={onNavigate} aria-label={t('header.home')} className="flex min-w-0 items-center gap-2.5 rounded-ui">
      <LogoMark />
      <span className="hidden min-w-0 leading-tight min-[400px]:block">
        <span className="block truncate text-[15px] font-bold text-ink">{t('meta.siteName')}</span>
        <span className="block truncate text-[11px] font-medium text-muted">{t('meta.tagline')}</span>
      </span>
    </Link>
  );
}
