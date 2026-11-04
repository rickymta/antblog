'use client';

import { forwardRef, useId, useState, type FormEvent } from 'react';
import { useRouter } from 'next/navigation';
import { useTranslation } from 'react-i18next';
import { Search } from 'lucide-react';

/** Ô tìm kiếm thanh đầu: Enter ⇒ /articles?q=…; gợi ý phím tắt Ctrl K (thanh đầu đăng ký phím). Không JS vẫn gửi được (GET /articles). */
export const SearchBox = forwardRef<HTMLInputElement, { onSubmitted?: () => void; className?: string; showHint?: boolean }>(
  function SearchBox({ onSubmitted, className = '', showHint = true }, ref) {
    const { t } = useTranslation();
    const router = useRouter();
    const [value, setValue] = useState('');
    const inputId = useId();

    const submit = (e: FormEvent) => {
      e.preventDefault();
      const q = value.trim();
      router.push(q ? `/articles?q=${encodeURIComponent(q)}` : '/articles');
      setValue('');
      onSubmitted?.();
    };

    return (
      <form role="search" action="/articles" method="get" onSubmit={submit} className={`relative ${className}`}>
        <label className="sr-only" htmlFor={inputId}>
          {t('header.searchLabel')}
        </label>
        <Search aria-hidden="true" className="pointer-events-none absolute top-1/2 left-3 h-4 w-4 -translate-y-1/2 text-muted" />
        <input
          ref={ref}
          id={inputId}
          name="q"
          type="search"
          value={value}
          onChange={(e) => setValue(e.target.value)}
          placeholder={t('header.searchPlaceholder')}
          autoComplete="off"
          enterKeyHint="search"
          className={`h-9 w-full rounded-ui border border-line bg-page pl-9 text-sm text-ink placeholder:text-muted focus:border-primary focus:bg-surface focus:outline-none focus:ring-2 focus:ring-primary/30 ${showHint ? 'pr-16' : 'pr-3'}`}
        />
        {showHint && (
          <kbd className="pointer-events-none absolute top-1/2 right-2 -translate-y-1/2 rounded border border-line bg-surface px-1.5 py-0.5 font-sans text-[10px] font-semibold text-muted">
            {t('header.shortcutHint')}
          </kbd>
        )}
      </form>
    );
  },
);
