// Địa chỉ site + tiện ích SEO dùng ở máy chủ (metadata, JSON-LD, sitemap, robots).

export const SITE_NAME = 'Tech Knowledge Hub';

/** URL gốc của trang đọc, không có `/` cuối. NEXT_PUBLIC_* được nhúng lúc build ⇒ đặt biến này khi build ảnh Docker. */
export const SITE_URL = (process.env.NEXT_PUBLIC_SITE_URL || 'https://antblog.xyz').replace(/\/+$/, '');

/** `SITE_NOINDEX=true` ⇒ chặn lập chỉ mục toàn site (staging). Đọc lúc chạy — trang/robots là động. */
export function siteNoindex(): boolean {
  return (process.env.SITE_NOINDEX ?? '').trim().toLowerCase() === 'true';
}

/** Đường dẫn tương đối (`/uploads/...`, `/articles/x`) ⇒ URL tuyệt đối theo SITE_URL; URL http(s) giữ nguyên. */
export function absoluteUrl(pathOrUrl: string): string {
  if (/^https?:\/\//i.test(pathOrUrl)) return pathOrUrl;
  return new URL(pathOrUrl.startsWith('/') ? pathOrUrl : `/${pathOrUrl}`, `${SITE_URL}/`).toString();
}

/** Ảnh Open Graph mặc định của site (app/opengraph-image.tsx). */
export const DEFAULT_OG_IMAGE = '/opengraph-image';

/** Logo dạng PNG cho JSON-LD Organization / manifest (app/logo.png/route.tsx). */
export const LOGO_PATH = '/logo.png';

export const OG_LOCALE = { vi: 'vi_VN', en: 'en_US' } as const;

/** Nguồn RSS 2.0 (app/rss.xml/route.ts) — 50 bài mới nhất. */
export const RSS_PATH = '/rss.xml';

/**
 * Liên kết khám phá RSS cho `alternates.types` — Next gộp metadata NÔNG: trang con đặt `alternates` (canonical) là thay
 * hẳn của layout ⇒ trang nào đặt canonical cũng phải mang lại khối này (lib/metadata.ts làm sẵn).
 */
export const RSS_ALTERNATE = { 'application/rss+xml': [{ url: RSS_PATH, title: SITE_NAME }] };

/** Mã xác minh Search Console / Bing Webmaster — đọc lúc chạy (cùng một ảnh Docker cho mọi môi trường). */
export function siteVerification(): { google?: string; bing?: string } {
  const google = (process.env.GOOGLE_SITE_VERIFICATION ?? '').trim();
  const bing = (process.env.BING_SITE_VERIFICATION ?? '').trim();
  return { ...(google ? { google } : {}), ...(bing ? { bing } : {}) };
}
