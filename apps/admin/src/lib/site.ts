// Địa chỉ trang đọc (apps/web) để mở bài viết. Đổi bằng VITE_SITE_URL khi build.
// Không đặt ⇒ dev server trang đọc; đặt rỗng (bản Docker) ⇒ cùng origin vì nginx phục vụ cả hai trang.
const envUrl = import.meta.env.VITE_SITE_URL as string | undefined;
export const SITE_URL = envUrl === undefined ? 'http://127.0.0.1:5173' : envUrl.trim().replace(/\/+$/, '');

export function siteArticleUrl(slug: string): string {
  return `${SITE_URL}/articles/${encodeURIComponent(slug)}`;
}

export const SLUG_RE = /^[a-z0-9]+(?:-[a-z0-9]+)*$/;
export const HTTP_URL_RE = /^https?:\/\/\S+$/i;
