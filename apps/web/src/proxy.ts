// Proxy (Next 16, Node.js runtime): trả 404 KHI CHƯA RENDER cho bài / chủ đề / lộ trình không tồn tại.
//
// Vì sao không chỉ dùng notFound() trong trang: notFound() ném trong page lồng nhau ⇒ lỗi lọt tới "shell" của React
// (không có error boundary phía máy chủ) ⇒ Next trả 404 nhưng HTML là khung rỗng `<html id="__next_error__">`, nội dung
// 404 chỉ hiện sau khi chạy JavaScript — trình thu thập không chạy JS thấy trang trắng. Next khuyên kiểm ở Proxy:
// slug không có ⇒ rewrite sang một đường dẫn KHÔNG khớp route nào ⇒ Next render `app/not-found.tsx` gốc ngay trên máy chủ
// (mã 404, nội dung đầy đủ trong HTML; câu báo riêng chọn theo URL đang mở — lib/notFoundKind.ts).
//
// Kiểm bằng `HEAD` tới API nội bộ, nhớ kết quả trong bộ nhớ (có: 60 giây, không có: 15 giây). API lỗi / quá thời gian ⇒
// cho đi tiếp như cũ (trang tự gọi notFound() — vẫn đúng mã 404, chỉ là nội dung dựng ở trình duyệt).
import { NextResponse, type NextRequest } from 'next/server';
import { NOT_FOUND_REWRITE_PATH } from './lib/notFoundKind';

const API_BASE = `${(process.env.API_INTERNAL_URL || 'http://127.0.0.1:4000').replace(/\/+$/, '')}/api/v1`;

/** Tiền tố URL trang đọc ⇒ tài nguyên API tương ứng. */
const SECTIONS: Record<string, string> = {
  articles: 'articles',
  topics: 'categories',
  'learning-paths': 'learning-paths',
};

const TTL_FOUND_MS = 60_000;
const TTL_MISSING_MS = 15_000;
const MAX_ENTRIES = 2_000;
const memo = new Map<string, { exists: boolean; until: number }>();

async function exists(api: string, slug: string): Promise<boolean | null> {
  const key = `${api}/${slug}`;
  const now = Date.now();
  const hit = memo.get(key);
  if (hit && hit.until > now) return hit.exists;
  try {
    const res = await fetch(`${API_BASE}/${api}/${encodeURIComponent(slug)}`, {
      method: 'HEAD',
      cache: 'no-store',
      signal: AbortSignal.timeout(2_000),
    });
    if (res.status !== 404 && !res.ok) return null;
    const found = res.ok;
    if (memo.size >= MAX_ENTRIES) memo.clear();
    memo.set(key, { exists: found, until: now + (found ? TTL_FOUND_MS : TTL_MISSING_MS) });
    return found;
  } catch {
    return null;
  }
}

export async function proxy(request: NextRequest) {
  const [, section, rawSlug] = request.nextUrl.pathname.split('/');
  const api = section ? SECTIONS[section] : undefined;
  if (!api || !rawSlug) return NextResponse.next();
  let slug: string;
  try {
    slug = decodeURIComponent(rawSlug);
  } catch {
    slug = rawSlug;
  }
  if ((await exists(api, slug)) !== false) return NextResponse.next();
  return NextResponse.rewrite(new URL(`${NOT_FOUND_REWRITE_PATH}/${section}`, request.url));
}

export const config = {
  // Chỉ trang chi tiết (một đoạn sau tiền tố). Ảnh OG `/og/articles/...` không đi qua đây.
  matcher: ['/articles/:slug', '/topics/:slug', '/learning-paths/:slug'],
};
