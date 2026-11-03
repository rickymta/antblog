import { Outlet, ScrollRestoration } from 'react-router';
import { useTranslation } from 'react-i18next';
import { useVisitTracking } from '../../hooks/useVisitTracking';
import { Footer } from './Footer';
import { Header } from './Header';

export function Layout() {
  const { t } = useTranslation();
  useVisitTracking();
  return (
    <div className="flex min-h-dvh flex-col overflow-x-clip">
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
      <Header />
      <main id="main" tabIndex={-1} className="flex-1 focus:outline-none">
        <Outlet />
      </main>
      <Footer />
      <ScrollRestoration />
    </div>
  );
}
