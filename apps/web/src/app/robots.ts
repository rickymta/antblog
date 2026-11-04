import type { MetadataRoute } from 'next';
import { SITE_URL, siteNoindex } from '../lib/site';

// Đọc SITE_NOINDEX lúc chạy (không chốt lúc build) ⇒ cùng một ảnh Docker dùng được cho staging lẫn production.
export const dynamic = 'force-dynamic';

export default function robots(): MetadataRoute.Robots {
  if (siteNoindex()) return { rules: [{ userAgent: '*', disallow: '/' }] };
  return {
    rules: [{ userAgent: '*', allow: '/', disallow: ['/login', '/register', '/me', '/api/'] }],
    sitemap: `${SITE_URL}/sitemap.xml`,
    host: SITE_URL,
  };
}
