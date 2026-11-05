import covers from './article-covers.json';

/** CMS images take precedence; reviewed local artwork fills articles without a cover. */
export function articleCover(article: { slug?: string; coverImageUrl?: string }): string | undefined {
  if (article.coverImageUrl) return article.coverImageUrl;
  return article.slug && Object.hasOwn(covers, article.slug)
    ? `/images/articles/${article.slug}.webp`
    : undefined;
}
