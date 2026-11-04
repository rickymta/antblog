// Gọi API công khai từ Server Components (máy chủ Next → API nội bộ). Trình duyệt vẫn dùng `api` của @tkh/shared (URL tương đối).
import { notFound } from 'next/navigation';
import { connection } from 'next/server';
import {
  ApiError,
  type ApiErrorBody,
  type ArticleCard,
  type ArticleDetail,
  type ArticleQuery,
  type Category,
  type LearningPathCard,
  type LearningPathDetail,
  type Level,
  type Page,
  type SiteSummary,
  type SitemapData,
  type TagCount,
} from '@tkh/shared';

/** Địa chỉ API phía máy chủ, mặc định API chạy cùng máy. */
const API_BASE = `${(process.env.API_INTERNAL_URL || 'http://127.0.0.1:4000').replace(/\/+$/, '')}/api/v1`;

/** Dữ liệu công khai được cache tối đa 60 giây (bài mới/sửa hiện ra trong vòng 1 phút). */
const DEFAULT_REVALIDATE = 60;

type Query = Record<string, string | number | undefined | null>;

function qs(query?: Query): string {
  if (!query) return '';
  const p = new URLSearchParams();
  for (const [k, v] of Object.entries(query)) if (v !== undefined && v !== null && v !== '') p.set(k, String(v));
  const s = p.toString();
  return s ? `?${s}` : '';
}

async function get<T>(path: string, query?: Query, revalidate = DEFAULT_REVALIDATE): Promise<T> {
  // Không bao giờ gọi API lúc `next build` (ảnh Docker build không có API) — mọi trang dùng dữ liệu là trang động.
  await connection();
  const res = await fetch(`${API_BASE}${path}${qs(query)}`, {
    headers: { Accept: 'application/json' },
    next: { revalidate, tags: ['api'] },
  });
  const text = await res.text();
  let data: unknown;
  try {
    data = text ? JSON.parse(text) : undefined;
  } catch {
    data = undefined;
  }
  if (!res.ok) throw new ApiError(res.status, data as ApiErrorBody | undefined);
  return data as T;
}

const enc = encodeURIComponent;

export const serverApi = {
  summary: () => get<SiteSummary>('/site/summary'),
  sitemap: () => get<SitemapData>('/site/sitemap', undefined, 300),
  categories: () => get<Category[]>('/categories', undefined, 300),
  category: (slug: string) => get<Category>(`/categories/${enc(slug)}`),
  articles: (query: ArticleQuery = {}) => get<Page<ArticleCard>>('/articles', { ...query }),
  trending: (limit = 4) => get<ArticleCard[]>('/articles/trending', { limit }),
  article: (slug: string) => get<ArticleDetail>(`/articles/${enc(slug)}`),
  related: (slug: string, limit = 3) => get<ArticleCard[]>(`/articles/${enc(slug)}/related`, { limit }),
  popularTags: (limit = 30) => get<TagCount[]>('/tags/popular', { limit }, 300),
  learningPaths: (query: { level?: Level; q?: string } = {}) => get<LearningPathCard[]>('/learning-paths', { ...query }),
  learningPath: (slug: string) => get<LearningPathDetail>(`/learning-paths/${enc(slug)}`),
};

/** API trả 404 ⇒ trang 404 của Next (mã HTTP 404); lỗi khác ném tiếp cho error boundary. */
export async function orNotFound<T>(promise: Promise<T>): Promise<T> {
  try {
    return await promise;
  } catch (e) {
    if (e instanceof ApiError && e.status === 404) notFound();
    throw e;
  }
}

/**
 * Dùng trong generateMetadata: 404 ⇒ `null` (trang tự gọi notFound() — ném notFound() trong generateMetadata làm Next
 * trả HTML rỗng, nội dung 404 chỉ hiện sau khi chạy JavaScript).
 */
export async function orNull<T>(promise: Promise<T>): Promise<T | null> {
  try {
    return await promise;
  } catch (e) {
    if (e instanceof ApiError && e.status === 404) return null;
    throw e;
  }
}

/** Khối phụ (bài liên quan, thẻ, danh sách ở trang chủ…): lỗi ⇒ `null`, trang vẫn hiển thị phần còn lại. */
export async function settle<T>(promise: Promise<T>): Promise<T | null> {
  try {
    return await promise;
  } catch (e) {
    console.error('[server-api]', e instanceof Error ? e.message : e);
    return null;
  }
}
