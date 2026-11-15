import type { MetadataRoute } from 'next';
import type { SitemapData } from '@tkh/shared';
import { serverApi, settle } from '../lib/server-api';
import { glossaryPath } from '../lib/glossary';
import { absoluteUrl } from '../lib/site';

// Dữ liệu lấy từ API lúc chạy (cache 5 phút ở tầng fetch) — không dựng lúc build vì ảnh Docker build không có API.
export const dynamic = 'force-dynamic';

/** API chưa có `GET /site/sitemap` (bản cũ) ⇒ dựng tạm từ các endpoint danh sách công khai. */
async function fallbackData(): Promise<SitemapData> {
  const [categories, paths] = await Promise.all([serverApi.categories(), serverApi.learningPaths()]);
  const articles: SitemapData['articles'] = [];
  for (let page = 1; page <= 50; page++) {
    const res = await serverApi.articles({ page, pageSize: 50, sort: 'latest' });
    articles.push(...res.items.map((a) => ({ slug: a.slug, updatedAt: a.updatedAt })));
    if (res.items.length === 0 || page * res.pageSize >= res.total) break;
  }
  return {
    articles,
    categories: categories.map((c) => ({ slug: c.slug })),
    learningPaths: paths.map((p) => ({ slug: p.slug, updatedAt: p.updatedAt })),
  };
}

const toDate = (iso: string | undefined) => {
  if (!iso) return undefined;
  const d = new Date(iso);
  return Number.isNaN(d.getTime()) ? undefined : d;
};

export default async function sitemap(): Promise<MetadataRoute.Sitemap> {
  let data: SitemapData;
  try {
    data = await serverApi.sitemap();
  } catch {
    data = await fallbackData();
  }

  // Từ điển là phần phụ: API lỗi ⇒ sitemap vẫn có các trang khác.
  const terms = (await settle(serverApi.glossary())) ?? [];
  const articles = data.articles.filter((a) => !a.noindex);
  const latest = articles.map((a) => toDate(a.updatedAt)).reduce<Date | undefined>((max, d) => (d && (!max || d > max) ? d : max), undefined);

  return [
    { url: absoluteUrl('/'), lastModified: latest, changeFrequency: 'daily', priority: 1 },
    { url: absoluteUrl('/articles'), lastModified: latest, changeFrequency: 'daily', priority: 0.8 },
    { url: absoluteUrl('/learning-paths'), changeFrequency: 'weekly', priority: 0.7 },
    { url: absoluteUrl('/about'), changeFrequency: 'yearly', priority: 0.3 },
    ...(terms.length
      ? [
          { url: absoluteUrl(glossaryPath()), changeFrequency: 'weekly' as const, priority: 0.5 },
          ...terms.map((t) => ({ url: absoluteUrl(glossaryPath(t.slug)), changeFrequency: 'monthly' as const, priority: 0.4 })),
        ]
      : []),
    ...data.categories.map((c) => ({
      url: absoluteUrl(`/topics/${encodeURIComponent(c.slug)}`),
      lastModified: toDate(c.updatedAt),
      changeFrequency: 'weekly' as const,
      priority: 0.6,
    })),
    ...data.learningPaths.map((p) => ({
      url: absoluteUrl(`/learning-paths/${encodeURIComponent(p.slug)}`),
      lastModified: toDate(p.updatedAt),
      changeFrequency: 'weekly' as const,
      priority: 0.7,
    })),
    ...articles.map((a) => ({
      url: absoluteUrl(`/articles/${encodeURIComponent(a.slug)}`),
      lastModified: toDate(a.updatedAt),
      changeFrequency: 'monthly' as const,
      priority: 0.8,
    })),
  ];
}
