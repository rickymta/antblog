import { pickLocalized } from '@tkh/shared';
import { brandImage } from '../../../../lib/og';
import { serverApi } from '../../../../lib/server-api';

/**
 * Ảnh Open Graph sinh từ tiêu đề bài — chỉ dùng cho bài KHÔNG có `seo.ogImageUrl` và không có ảnh bìa.
 * Bài không tồn tại ⇒ 404. Trình duyệt/CDN cache 1 giờ.
 */
export async function GET(_req: Request, { params }: { params: Promise<{ slug: string }> }) {
  const { slug } = await params;
  let article;
  try {
    article = await serverApi.article(slug);
  } catch {
    return new Response('Not found', { status: 404 });
  }
  const image = await brandImage({
    eyebrow: pickLocalized(article.category.name, 'vi').toUpperCase(),
    title: article.title,
    footer: `${article.author.displayName} · ${article.readingMinutes} phút đọc`,
  });
  image.headers.set('Cache-Control', 'public, max-age=3600, s-maxage=3600, stale-while-revalidate=86400');
  return image;
}
