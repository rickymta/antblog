// Lớp Tailwind dùng lặp lại — gom một chỗ để nút/ô nhập/thẻ đồng nhất mọi trang.

const btnBase =
  'inline-flex items-center justify-center gap-2 rounded-ui px-4 py-2 text-sm font-semibold transition-colors disabled:cursor-not-allowed disabled:opacity-50';

export const btnPrimary = `${btnBase} bg-primary text-white hover:bg-primary-hover`;
export const btnSecondary = `${btnBase} border border-line bg-surface text-ink hover:bg-surface-2`;
export const btnGhost = `${btnBase} text-ink hover:bg-surface-2`;
export const btnLink = 'inline-flex items-center gap-1 text-sm font-semibold text-primary-ink hover:underline underline-offset-4';

export const iconBtn =
  'inline-flex h-9 w-9 shrink-0 items-center justify-center rounded-ui text-muted transition-colors hover:bg-surface-2 hover:text-ink';

export const inputClass =
  'w-full rounded-ui border border-line bg-surface px-3 py-2 text-sm text-ink placeholder:text-muted focus:border-primary focus:outline-none focus:ring-2 focus:ring-primary/30 aria-[invalid=true]:border-red-500';

export const selectClass = `${inputClass} pr-8`;

export const card = 'rounded-card border border-line bg-surface';

export const container = 'mx-auto w-full max-w-7xl px-4 sm:px-6 lg:px-8';
