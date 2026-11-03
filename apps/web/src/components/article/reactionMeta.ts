import type { ReactionType } from '@tkh/shared';

/** Thứ tự, emoji và tên tiếng Anh của 4 loại reaction (A7). Nghĩa tiếng Việt lấy từ i18n `reactions.<type>`. */
export const REACTIONS: readonly { type: ReactionType; emoji: string; name: string }[] = [
  { type: 'love', emoji: '❤️', name: 'Love' },
  { type: 'helpful', emoji: '👍', name: 'Helpful' },
  { type: 'insightful', emoji: '💡', name: 'Insightful' },
  { type: 'inspiring', emoji: '🚀', name: 'Inspiring' },
];
