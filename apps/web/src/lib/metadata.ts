import type { Metadata } from 'next';
import type { Locale } from '@tkh/shared';
import { getServerI18n } from '../i18n/server';
import { absoluteUrl, DEFAULT_OG_IMAGE, OG_LOCALE, RSS_ALTERNATE, SITE_NAME } from './site';

interface PageMetaInput {
  /** Tiêu đề trang — ghép mẫu "%s · Tech Knowledge Hub". */
  title?: string;
  /** Tiêu đề đầy đủ, KHÔNG ghép tên site (vd. tiêu đề SEO do biên tập viên đặt cho bài). */
  absoluteTitle?: string;
  description: string;
  /** Đường dẫn chuẩn (canonical), bắt đầu bằng `/`, có thể kèm `?page=N`. */
  path: string;
  locale: Locale;
  /** Ảnh chia sẻ (URL tuyệt đối hoặc đường dẫn `/uploads/...`); vắng ⇒ ảnh mặc định của site. */
  image?: { url: string; alt?: string; width?: number; height?: number };
  noindex?: boolean;
  openGraph?: Metadata['openGraph'];
}

/**
 * Metadata đầy đủ cho một trang: description, canonical, Open Graph, Twitter.
 * `openGraph`/`twitter` của trang con THAY HẲN của layout (Next gộp nông) ⇒ luôn dựng đủ ở đây.
 * Chỉ đặt `robots` khi cần noindex — còn lại kế thừa layout gốc (SITE_NOINDEX).
 */
export function pageMetadata(input: PageMetaInput): Metadata {
  const shownTitle = input.absoluteTitle ?? (input.title ? `${input.title} · ${SITE_NAME}` : SITE_NAME);
  const image = input.image
    ? { ...input.image, url: absoluteUrl(input.image.url), alt: input.image.alt ?? shownTitle }
    : { url: absoluteUrl(DEFAULT_OG_IMAGE), width: 1200, height: 630, alt: SITE_NAME };
  const meta: Metadata = {
    title: input.absoluteTitle ? { absolute: input.absoluteTitle } : input.title,
    description: input.description,
    alternates: { canonical: input.path, types: RSS_ALTERNATE },
    openGraph: {
      type: 'website',
      siteName: SITE_NAME,
      locale: OG_LOCALE[input.locale],
      url: input.path,
      title: shownTitle,
      description: input.description,
      images: [image],
      ...input.openGraph,
    } as Metadata['openGraph'],
    twitter: {
      card: 'summary_large_image',
      title: shownTitle,
      description: input.description,
      images: [image.url],
    },
  };
  if (input.noindex) meta.robots = { index: false, follow: true };
  return meta;
}

/** Cắt mô tả về ≤ 160 ký tự theo ranh giới từ. */
export function clampDescription(text: string, max = 160): string {
  const clean = text.replace(/\s+/g, ' ').trim();
  if (clean.length <= max) return clean;
  const cut = clean.slice(0, max - 1);
  const space = cut.lastIndexOf(' ');
  return `${(space > max * 0.6 ? cut.slice(0, space) : cut).replace(/[\s,.;:–—-]+$/, '')}…`;
}

/** Metadata khi API trả 404 (trang sẽ gọi notFound()): tiêu đề "Không tìm thấy trang"; Next tự thêm noindex. */
export async function notFoundMetadata(): Promise<Metadata> {
  const { t } = await getServerI18n();
  return { title: t('notFound.title'), robots: { index: false, follow: true } };
}
