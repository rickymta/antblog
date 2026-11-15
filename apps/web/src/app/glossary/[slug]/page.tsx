import Link from 'next/link';
import type { Metadata } from 'next';
import { ArticleContent } from '@tkh/ui';
import { I18N_ENABLED } from '../../../i18n/config';
import { getServerI18n } from '../../../i18n/server';
import { definedTerm, glossaryPath } from '../../../lib/glossary';
import { breadcrumbList, JsonLd } from '../../../lib/jsonld';
import { clampDescription, notFoundMetadata, pageMetadata } from '../../../lib/metadata';
import { orNotFound, orNull, serverApi } from '../../../lib/server-api';
import { absoluteUrl, SITE_URL } from '../../../lib/site';
import { card, container } from '../../../lib/ui';

type Props = { params: Promise<{ slug: string }> };

/**
 * ISR như trang bài: dựng lần đầu khi có request rồi cache 5 phút; generateStaticParams trả [] ⇒ không dựng gì lúc
 * `next build` (ảnh Docker build không có API). Bật song ngữ ⇒ trang động (bỏ generateStaticParams).
 */
export const revalidate = 300;
export const generateStaticParams = I18N_ENABLED ? undefined : async (): Promise<{ slug: string }[]> => [];

export async function generateMetadata({ params }: Props): Promise<Metadata> {
  const { slug } = await params;
  const { t, locale } = await getServerI18n();
  const term = await orNull(serverApi.glossaryTerm(slug, locale));
  if (!term) return notFoundMetadata();
  return pageMetadata({
    title: `${term.term} · ${t('glossary.title')}`,
    description: clampDescription(term.shortDefinition),
    path: glossaryPath(term.slug),
    locale,
  });
}

export default async function GlossaryTermPage({ params }: Props) {
  const { slug } = await params;
  const { t, locale } = await getServerI18n();
  const term = await orNotFound(serverApi.glossaryTerm(slug, locale));
  const setName = t('glossary.title');
  const showEnglish = term.english && term.english.toLowerCase() !== term.term.toLowerCase();
  const aliases = term.aliases.filter((a) => a.toLowerCase() !== term.term.toLowerCase() && a.toLowerCase() !== term.english?.toLowerCase());

  return (
    <div className={container}>
      <JsonLd data={definedTerm(term, setName)} />
      <JsonLd
        data={breadcrumbList([
          { name: t('nav.home'), url: `${SITE_URL}/` },
          { name: setName, url: absoluteUrl(glossaryPath()) },
          { name: term.term, url: absoluteUrl(glossaryPath(term.slug)) },
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
            <Link href={glossaryPath()} className="inline-flex min-h-11 items-center hover:text-primary-ink hover:underline">
              {setName}
            </Link>
          </li>
          <li className="flex items-center gap-2">
            <span aria-hidden="true">/</span>
            <span aria-current="page" className="text-ink">
              {term.term}
            </span>
          </li>
        </ol>
      </nav>

      <article className="mx-auto max-w-3xl pt-4 pb-12">
        <p className="text-sm font-semibold tracking-wide text-primary-ink uppercase">{t('glossary.eyebrow')}</p>
        <h1 className="mt-2 text-3xl font-bold tracking-tight text-ink sm:text-4xl">{term.term}</h1>
        {showEnglish && (
          <p className="mt-2 text-base text-muted">
            {t('glossary.english')}: <span lang="en">{term.english}</span>
          </p>
        )}
        <p className="mt-5 text-lg leading-relaxed text-ink">{term.shortDefinition}</p>
        {aliases.length > 0 && (
          <p className="mt-4 text-sm text-muted">
            {t('glossary.aliases')}: {aliases.join(', ')}
          </p>
        )}

        {term.body && (
          <section aria-labelledby="glossary-body" className="mt-10">
            <h2 id="glossary-body" className="mb-4 text-xl font-bold text-ink">
              {t('glossary.explanation')}
            </h2>
            <ArticleContent markdown={term.body} />
          </section>
        )}

        <section aria-labelledby="glossary-related" className="mt-10">
          <h2 id="glossary-related" className="mb-4 text-xl font-bold text-ink">
            {t('glossary.relatedArticles')}
          </h2>
          {term.articles.length === 0 ? (
            <p className="text-sm text-muted">{t('glossary.noRelated')}</p>
          ) : (
            <ul className={`${card} divide-y divide-line`}>
              {term.articles.map((a) => (
                <li key={a.slug}>
                  <Link href={`/articles/${encodeURIComponent(a.slug)}`} className="flex min-h-11 items-center px-5 py-3 font-medium text-ink hover:bg-surface-2 hover:text-primary-ink">
                    {a.title}
                  </Link>
                </li>
              ))}
            </ul>
          )}
        </section>

        <p className="mt-10">
          <Link href={glossaryPath()} className="inline-flex min-h-11 items-center gap-2 text-sm font-semibold text-primary-ink hover:underline">
            <span aria-hidden="true">←</span> {t('glossary.backToList')}
          </Link>
        </p>
      </article>
    </div>
  );
}
