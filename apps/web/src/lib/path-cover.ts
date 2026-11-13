/** Reuse the wide topic artwork; topics without artwork use Cover's category fallback. */
export function pathCover(slug: string): string | undefined {
  const topic = slug === 'postgresql' ? 'database' : slug;
  const available = new Set([
    'architecture', 'business-analysis', 'csharp', 'csharp-advanced', 'csharp-beginner',
    'csharp-intermediate', 'database', 'design-pattern', 'devops', 'devops-aws',
    'devops-azure', 'flutter', 'go', 'go-advanced', 'go-beginner', 'go-intermediate',
    'location-routing', 'programming', 'python', 'python-advanced', 'python-beginner',
    'python-intermediate', 'rust', 'rust-advanced', 'rust-beginner', 'rust-intermediate',
    'security', 'software-design', 'system-design', 'testing',
  ]);
  return available.has(topic) ? `/images/categories/${topic}.webp` : undefined;
}
