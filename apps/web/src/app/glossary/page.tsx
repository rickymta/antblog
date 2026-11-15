import Link from 'next/link';
import type { Metadata } from 'next';
import { formatNumber } from '@tkh/shared';
import { getServerI18n } from '../../i18n/server';
import { definedTermSet, glossaryPath, groupByLetter } from '../../lib/glossary';
import { breadcrumbList, JsonLd } from '../../lib/jsonld';
import { pageMetadata } from '../../lib/metadata';
import { serverApi } from '../../lib/server-api';
import { absoluteUrl, SITE_URL } from '../../lib/site';
import { card, container } from '../../lib/ui';

/** Danh sách thuật ngữ đổi ít ⇒ cache 5 phút (cùng nhịp với `serverApi.glossary`). */
export const revalidate = 300;

export async function generateMetadata(): Promise<Metadata> {
  const { t, locale } = await getServerI18n();
  return pageMetadata({ title: t('glossary.title'), description: t('glossary.description'), path: glossaryPath(), locale });
}

/** Từ điển thuật ngữ A–Z (giai đoạn 5 — docs/content-dictionary/02-glossary-feature.md). */
export default async function GlossaryPage() {
  const { t, locale } = await getServerI18n();
  const terms = await serverApi.glossary(locale);
  const groups = groupByLetter(terms, locale);
  const title = t('glossary.title');

  return (
    <div className={container}>
      <JsonLd data={definedTermSet(title, t('glossary.description'), terms)} />
      <JsonLd
        data={breadcrumbList([
          { name: t('nav.home'), url: `${SITE_URL}/` },
          { name: title, url: absoluteUrl(glossaryPath()) },
        ])}
      />
      <nav aria-label={t('article.breadcrumb')} className="pt-4 text-sm text-muted">
        <ol className="flex flex-wrap items-center gap-x-2 gap-y-1">
          <li>
            <Link href="/" className="inline-flex min-h-11 items-center hover:text-primary-ink hover:underline">
              {t('nav.home')}
            </Link>
          </li>
          <li className="flex items-center gap-2">
            <span aria-hidden="true">/</span>
            <span aria-current="page" className="text-ink">
              {title}
            </span>
          </li>
        </ol>
      </nav>

      <header className="max-w-3xl pt-4 pb-8">
        <p className="text-sm font-semibold tracking-wide text-primary-ink uppercase">{t('glossary.eyebrow')}</p>
        <h1 className="mt-2 text-3xl font-bold tracking-tight text-ink sm:text-4xl">{title}</h1>
        <p className="mt-3 text-base leading-relaxed text-muted">{t('glossary.intro')}</p>
        {terms.length > 0 && <p className="mt-2 text-sm text-muted">{t('glossary.count', { n: formatNumber(terms.length, locale) })}</p>}
      </header>

      {terms.length === 0 ? (
        <p className={`${card} p-6 text-muted`}>{t('glossary.empty')}</p>
      ) : (
        <div className="space-y-10 pb-12">
          {groups.length > 1 && (
            <nav aria-label={t('glossary.letterNav')} className="sticky top-16 z-10 -mx-4 border-b border-line bg-page/95 px-4 py-2 backdrop-blur sm:mx-0 sm:rounded-card sm:border">
              <ul className="flex flex-wrap gap-1">
                {groups.map((g) => (
                  <li key={g.letter}>
                    <a href={`#letter-${g.letter === '#' ? 'other' : g.letter}`} className="inline-flex size-11 items-center justify-center rounded-ui text-sm font-semibold text-ink hover:bg-surface-2 hover:text-primary-ink">
                      {g.letter}
                    </a>
                  </li>
                ))}
              </ul>
            </nav>
          )}

          {groups.map((g) => {
            const id = `letter-${g.letter === '#' ? 'other' : g.letter}`;
            return (
              <section key={g.letter} aria-labelledby={id} className="scroll-mt-32">
                <h2 id={id} className="mb-4 border-b border-line pb-2 text-2xl font-bold text-ink">
                  {g.letter}
                </h2>
                <dl className="grid gap-4 md:grid-cols-2">
                  {g.terms.map((term) => (
                    <div key={term.slug} className={`${card} p-5`}>
                      <dt>
                        <Link href={glossaryPath(term.slug)} className="text-lg font-semibold text-ink hover:text-primary-ink hover:underline">
                          {term.term}
                        </Link>
                        {term.english && term.english.toLowerCase() !== term.term.toLowerCase() && (
                          <span className="ml-2 text-sm text-muted" lang="en">
                            {term.english}
                          </span>
                        )}
                      </dt>
                      <dd className="mt-2 text-sm leading-relaxed text-muted">{term.shortDefinition}</dd>
                    </div>
                  ))}
                </dl>
              </section>
            );
          })}
        </div>
      )}
    </div>
  );
}
