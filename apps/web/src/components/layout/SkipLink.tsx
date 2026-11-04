'use client';

import { useTranslation } from 'react-i18next';

/** "Bỏ qua, tới nội dung chính" — hiện khi focus bằng bàn phím. */
export function SkipLink() {
  const { t } = useTranslation();
  return (
    <a
      href="#main"
      onClick={(e) => {
        e.preventDefault();
        const main = document.getElementById('main');
        main?.focus();
        main?.scrollIntoView();
      }}
      className="sr-only z-50 rounded-ui bg-primary px-4 py-2 font-semibold text-white focus:not-sr-only focus:fixed focus:top-3 focus:left-3"
    >
      {t('nav.skipToContent')}
    </a>
  );
}
