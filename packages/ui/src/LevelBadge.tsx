import type { Level } from '@tkh/shared';

// Màu cấp độ thống nhất mọi nơi (docs/ui-guide.md §1): Beginner emerald, Intermediate indigo, Advanced violet.
const LEVEL_CLASS: Record<Level, string> = {
  beginner: 'bg-emerald-100 text-emerald-800 dark:bg-emerald-500/15 dark:text-emerald-300',
  intermediate: 'bg-indigo-100 text-indigo-800 dark:bg-indigo-500/20 dark:text-indigo-200',
  advanced: 'bg-violet-100 text-violet-800 dark:bg-violet-500/20 dark:text-violet-200',
};

/** Nhãn cấp độ; chữ hiển thị truyền vào qua `label` (mỗi app tự dịch). */
export function LevelBadge({ level, label, className = '' }: { level: Level; label: string; className?: string }) {
  return (
    <span
      className={`inline-flex items-center rounded-ui px-2 py-0.5 text-xs font-semibold whitespace-nowrap ${LEVEL_CLASS[level]} ${className}`}
    >
      {label}
    </span>
  );
}
