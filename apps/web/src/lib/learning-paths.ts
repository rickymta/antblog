import { comparePathOrder, LEVELS, type LearningPathCard } from '@tkh/shared';

/** Parts of one topic in learning order: explicit priority first, then foundations before advanced, then by slug so "-phan-02" follows its base part. */
export function orderTopicParts<T extends Pick<LearningPathCard, 'level' | 'slug' | 'priority'>>(paths: T[]): T[] {
  return [...paths].sort(comparePathOrder);
}

/** Across topics (home, path index): available paths first, then foundations before advanced study. Priority only orders parts within one topic. */
export function orderLearningPaths(paths: LearningPathCard[]): LearningPathCard[] {
  return [...paths].sort((a, b) =>
    Number(b.articleCount > 0) - Number(a.articleCount > 0) ||
    LEVELS.indexOf(a.level) - LEVELS.indexOf(b.level),
  );
}
