import { LEVELS, type LearningPathCard } from '@tkh/shared';

/** Available paths first, then foundations before advanced study. Keep API order within each level. */
/** Parts of one topic in learning order: foundations first, then by slug so "-phan-02" follows its base part. */
export function orderTopicParts<T extends Pick<LearningPathCard, 'level' | 'slug'>>(paths: T[]): T[] {
  return [...paths].sort((a, b) =>
    LEVELS.indexOf(a.level) - LEVELS.indexOf(b.level) || a.slug.localeCompare(b.slug, 'en', { numeric: true }),
  );
}

export function orderLearningPaths(paths: LearningPathCard[]): LearningPathCard[] {
  return [...paths].sort((a, b) =>
    Number(b.articleCount > 0) - Number(a.articleCount > 0) ||
    LEVELS.indexOf(a.level) - LEVELS.indexOf(b.level),
  );
}
