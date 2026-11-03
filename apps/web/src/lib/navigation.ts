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
