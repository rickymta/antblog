import type { GlossaryTerm } from '@tkh/shared';
import { absoluteUrl } from './site';

export const glossaryPath = (slug?: string) => (slug ? `/glossary/${encodeURIComponent(slug)}` : '/glossary');

/** Chữ cái để nhóm thuật ngữ: bỏ dấu ("Đ" → "D"); không phải chữ Latin ⇒ "#". */
export function termLetter(term: string): string {
  const first = term
    .trim()
    .normalize('NFD')
    .replace(/[̀-ͯ]/g, '')
    .replace(/[đĐ]/g, 'D')
    .charAt(0)
    .toUpperCase();
  return /[A-Z]/.test(first) ? first : '#';
}

/** Nhóm theo chữ cái, mỗi nhóm sắp theo thứ tự chữ cái tiếng Việt. Nhóm "#" đứng cuối. */
export function groupByLetter(terms: readonly GlossaryTerm[], locale: string): Array<{ letter: string; terms: GlossaryTerm[] }> {
  const groups = new Map<string, GlossaryTerm[]>();
  for (const t of terms) {
    const letter = termLetter(t.term);
    groups.set(letter, [...(groups.get(letter) ?? []), t]);
  }
  const collator = new Intl.Collator(locale, { sensitivity: 'base' });
  return [...groups.entries()]
    .sort(([a], [b]) => (a === '#' ? 1 : b === '#' ? -1 : a.localeCompare(b)))
    .map(([letter, list]) => ({ letter, terms: list.sort((x, y) => collator.compare(x.term, y.term)) }));
}

/** JSON-LD schema.org DefinedTermSet cho trang /glossary. */
export function definedTermSet(name: string, description: string, terms: readonly GlossaryTerm[]) {
  const url = absoluteUrl(glossaryPath());
  return {
    '@context': 'https://schema.org',
    '@type': 'DefinedTermSet',
    '@id': url,
    name,
    description,
    url,
    hasDefinedTerm: terms.map((t) => ({
      '@type': 'DefinedTerm',
      name: t.term,
      description: t.shortDefinition,
      url: absoluteUrl(glossaryPath(t.slug)),
      ...(t.english ? { alternateName: t.english } : {}),
    })),
  };
}

/** JSON-LD schema.org DefinedTerm cho trang /glossary/<slug>. */
export function definedTerm(term: GlossaryTerm, setName: string) {
  const alternates = [...new Set([term.english, ...term.aliases].filter((s): s is string => !!s && s !== term.term))];
  return {
    '@context': 'https://schema.org',
    '@type': 'DefinedTerm',
    name: term.term,
    description: term.shortDefinition,
    url: absoluteUrl(glossaryPath(term.slug)),
    termCode: term.slug,
    ...(alternates.length ? { alternateName: alternates } : {}),
    inDefinedTermSet: { '@type': 'DefinedTermSet', '@id': absoluteUrl(glossaryPath()), name: setName, url: absoluteUrl(glossaryPath()) },
  };
}
