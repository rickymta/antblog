import covers from './path-covers.json';

export function learningPathCover(slug: string): string | undefined {
  return Object.hasOwn(covers, slug) ? `/images/learning-paths/${slug}.webp` : undefined;
}

/** Wide topic artwork, separate from each learning path's dedicated illustration. */
export function pathCover(slug: string): string | undefined {
  const topic = slug;
  const available = new Set([
    'architecture', 'business-analysis', 'csharp', 'csharp-advanced', 'csharp-beginner',
    'csharp-intermediate', 'database', 'design-pattern', 'devops', 'devops-aws',
    'devops-azure', 'flutter', 'go', 'go-advanced', 'go-beginner', 'go-intermediate',
    'location-routing', 'programming', 'python', 'python-advanced', 'python-beginner',
    'python-intermediate', 'rust', 'rust-advanced', 'rust-beginner', 'rust-intermediate',
    'security', 'software-design', 'system-design', 'testing', 'postgresql', 'data-analysis',
  ]);
  return available.has(topic) ? `/images/categories/${topic}.webp` : undefined;
}
