import { pickLocalized } from '@tkh/shared';
import { createI18n, DEFAULT_LOCALE } from '../../i18n/resources';
import { serverApi } from '../../lib/server-api';
import { absoluteUrl, RSS_PATH, SITE_NAME, SITE_URL } from '../../lib/site';

// RSS 2.0: 50 bài đã xuất bản mới nhất. Lấy từ API lúc chạy (ảnh Docker build không có API) — fetch cache 10 phút,
// trình duyệt/CDN cache 10 phút qua Cache-Control.
export const dynamic = 'force-dynamic';

const CACHE_SECONDS = 600;

/** Thoát ký tự đặc biệt XML (nội dung văn bản và giá trị thuộc tính). */
function xml(text: string): string {
  return text
    .replace(/[\u0000-\u0008\u000B\u000C\u000E-\u001F]/g, '')
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&apos;');
}

const rfc822 = (iso: string | undefined) => {
  const d = iso ? new Date(iso) : new Date();
  return (Number.isNaN(d.getTime()) ? new Date() : d).toUTCString();
};

export async function GET() {
  let page;
  try {
    page = await serverApi.feed();
  } catch (e) {
    console.error('[rss]', e instanceof Error ? e.message : e);
    return new Response('RSS tạm thời không khả dụng.', {
      status: 503,
      headers: { 'Content-Type': 'text/plain; charset=utf-8', 'Retry-After': '60', 'Cache-Control': 'no-store' },
    });
  }
  const t = createI18n(DEFAULT_LOCALE).getFixedT(DEFAULT_LOCALE);
  const items = page.items;
  const lastBuild = items[0]?.publishedAt;
  const feedUrl = absoluteUrl(RSS_PATH);

  const body = `<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0" xmlns:atom="http://www.w3.org/2005/Atom">
  <channel>
    <title>${xml(SITE_NAME)}</title>
    <link>${xml(`${SITE_URL}/`)}</link>
    <description>${xml(t('meta.description'))}</description>
    <language>vi</language>
    <lastBuildDate>${rfc822(lastBuild)}</lastBuildDate>
    <ttl>${CACHE_SECONDS / 60}</ttl>
    <atom:link href="${xml(feedUrl)}" rel="self" type="application/rss+xml" />
${items
  .map((a) => {
    const url = absoluteUrl(`/articles/${encodeURIComponent(a.slug)}`);
    return `    <item>
      <title>${xml(a.title)}</title>
      <link>${xml(url)}</link>
      <guid isPermaLink="true">${xml(url)}</guid>
      <pubDate>${rfc822(a.publishedAt)}</pubDate>
      <description>${xml(a.summary || a.title)}</description>
      <category>${xml(pickLocalized(a.category.name, DEFAULT_LOCALE))}</category>
    </item>`;
  })
  .join('\n')}
  </channel>
</rss>
`;
  return new Response(body, {
    headers: {
      'Content-Type': 'application/rss+xml; charset=utf-8',
      'Cache-Control': `public, max-age=${CACHE_SECONDS}, s-maxage=${CACHE_SECONDS}, stale-while-revalidate=${CACHE_SECONDS}`,
    },
  });
}
