export const MAIN_LINKS = [
  { to: '/', label: 'nav.home', end: true },
  { to: '/articles', label: 'nav.articles', end: false },
  { to: '/learning-paths', label: 'nav.paths', end: false },
  { to: '/about', label: 'nav.about', end: false },
] as const;

/** Liên kết đang ở trang hiện tại (thay NavLink của React Router). `end` ⇒ chỉ khớp đúng đường dẫn. */
export function isActivePath(pathname: string, to: string, end: boolean): boolean {
  if (end || to === '/') return pathname === to;
  return pathname === to || pathname.startsWith(`${to}/`);
}
