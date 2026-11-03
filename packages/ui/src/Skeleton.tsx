/** Khối giữ chỗ khi đang tải. Ẩn với trình đọc màn hình — vùng chứa nên có `aria-busy`. */
export function Skeleton({ className = '' }: { className?: string }) {
  return <div aria-hidden="true" className={`animate-pulse rounded-ui bg-surface-2 dark:bg-slate-700/60 ${className}`} />;
}
