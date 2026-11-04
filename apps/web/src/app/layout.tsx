import type { Metadata, Viewport } from 'next';
import type { ReactNode } from 'react';
import './globals.css';
import { fontVariables } from './fonts';
import { Providers } from '../components/Providers';
import { Footer } from '../components/layout/Footer';
import { Header } from '../components/layout/Header';
import { SkipLink } from '../components/layout/SkipLink';
import { getServerI18n } from '../i18n/server';
import { serverApi, settle } from '../lib/server-api';
import { DEFAULT_OG_IMAGE, OG_LOCALE, RSS_ALTERNATE, SITE_NAME, SITE_URL, siteNoindex, siteVerification } from '../lib/site';

// Áp theme trước khi trang hiển thị để không nháy màu: localStorage 'tkh-theme' = light|dark, không có thì theo hệ điều hành.
const THEME_SCRIPT = `try{var t=localStorage.getItem('tkh-theme');if(t==='dark'||(!t&&matchMedia('(prefers-color-scheme: dark)').matches))document.documentElement.classList.add('dark')}catch(e){}`;

export async function generateMetadata(): Promise<Metadata> {
  const { t, locale } = await getServerI18n();
  const noindex = siteNoindex();
  const verification = siteVerification();
  return {
    metadataBase: new URL(SITE_URL),
    title: { default: `${SITE_NAME} — ${t('meta.tagline')}`, template: `%s · ${SITE_NAME}` },
    description: t('meta.description'),
    applicationName: SITE_NAME,
    // Trang con chỉ đặt robots khi cần noindex (ghi đè khối này); SITE_NOINDEX=true (staging) chặn toàn site.
    robots: noindex
      ? { index: false, follow: false, googleBot: { index: false, follow: false } }
      : {
          index: true,
          follow: true,
          googleBot: { index: true, follow: true, 'max-image-preview': 'large', 'max-snippet': -1, 'max-video-preview': -1 },
        },
    alternates: { types: RSS_ALTERNATE },
    ...(verification.google || verification.bing
      ? {
          verification: {
            ...(verification.google ? { google: verification.google } : {}),
            ...(verification.bing ? { other: { 'msvalidate.01': verification.bing } } : {}),
          },
        }
      : {}),
    openGraph: {
      type: 'website',
      siteName: SITE_NAME,
      locale: OG_LOCALE[locale],
      title: SITE_NAME,
      description: t('meta.description'),
      images: [{ url: DEFAULT_OG_IMAGE, width: 1200, height: 630, alt: SITE_NAME }],
    },
    twitter: { card: 'summary_large_image', title: SITE_NAME, description: t('meta.description'), images: [DEFAULT_OG_IMAGE] },
    formatDetection: { telephone: false, email: false, address: false },
  };
}

export const viewport: Viewport = {
  width: 'device-width',
  initialScale: 1,
  themeColor: [
    { media: '(prefers-color-scheme: light)', color: '#F8FAFC' },
    { media: '(prefers-color-scheme: dark)', color: '#0F172A' },
  ],
};

export default async function RootLayout({ children }: { children: ReactNode }) {
  const { locale } = await getServerI18n();
  // Danh sách chủ đề cho menu "Chủ đề", ngăn kéo điện thoại và chân trang (cache 5 phút); lỗi ⇒ menu báo chưa tải được.
  const categories = await settle(serverApi.categories());
  return (
    <html lang={locale} className={fontVariables} suppressHydrationWarning>
      <head>
        <script dangerouslySetInnerHTML={{ __html: THEME_SCRIPT }} />
      </head>
      <body>
        <Providers locale={locale}>
          <div className="flex min-h-dvh flex-col overflow-x-clip">
            <SkipLink />
            <Header categories={categories} />
            <main id="main" tabIndex={-1} className="flex-1 focus:outline-none">
              {children}
            </main>
            <Footer categories={categories} year={new Date().getFullYear()} />
          </div>
        </Providers>
      </body>
    </html>
  );
}
