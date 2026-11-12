import { LEVELS, type LearningPathCard } from '@tkh/shared';

/** Available paths first, then foundations before advanced study. Keep API order within each level. */
export function orderLearningPaths(paths: LearningPathCard[]): LearningPathCard[] {
  return [...paths].sort((a, b) =>
    Number(b.articleCount > 0) - Number(a.articleCount > 0) ||
    LEVELS.indexOf(a.level) - LEVELS.indexOf(b.level),
  );
}
