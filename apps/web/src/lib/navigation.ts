/** Chỉ chấp nhận đường dẫn nội bộ ("/..." nhưng không phải "//host") để tránh chuyển hướng ra ngoài. */
export function safeNext(value: string | null | undefined, fallback = '/'): string {
  if (!value || !value.startsWith('/') || value.startsWith('//') || value.startsWith('/\\')) return fallback;
  if (value.startsWith('/login') || value.startsWith('/register')) return fallback;
  return value;
}

export function loginHref(next: string): string {
  return `/login?next=${encodeURIComponent(next)}`;
}

export function registerHref(next: string): string {
  return `/register?next=${encodeURIComponent(next)}`;
}

/** Liên kết trang N giữ nguyên các tham số lọc khác; trang 1 bỏ `page`. */
export function pageHref(basePath: string, params: Record<string, string>, page: number): string {
  const next = new URLSearchParams(params);
  if (page > 1) next.set('page', String(page));
  else next.delete('page');
  const s = next.toString();
  return s ? `${basePath}?${s}` : basePath;
}
