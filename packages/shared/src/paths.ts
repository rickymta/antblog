import { LEVELS, type LearningPathCard } from './types';

type Orderable = Pick<LearningPathCard, 'level' | 'slug' | 'priority'>;

/**
 * Thứ tự các phần (lộ trình) trong cùng một chủ đề: `priority` tăng dần (vắng ⇒ sau mọi phần có số) → cấp độ → slug
 * (để "-phan-02" đứng sau phần gốc). Phải khớp `comparePathOrder` ở api/src/services/articles.ts.
 */
export function comparePathOrder(a: Orderable, b: Orderable): number {
  const pa = a.priority ?? Number.POSITIVE_INFINITY;
  const pb = b.priority ?? Number.POSITIVE_INFINITY;
  return (pa === pb ? 0 : pa < pb ? -1 : 1)
    || LEVELS.indexOf(a.level) - LEVELS.indexOf(b.level)
    || a.slug.localeCompare(b.slug, 'en', { numeric: true });
}
