import type { Metadata } from 'next';
import Link from 'next/link';
import { ArrowRight, BookOpen, Braces, Database, Info, MapPin, Route, Workflow } from 'lucide-react';
import { PageBanner } from '../../components/common/PageBanner';
import { getServerI18n } from '../../i18n/server';
import { pageMetadata } from '../../lib/metadata';
import { btnLink, btnPrimary, btnSecondary, card, container } from '../../lib/ui';

export async function generateMetadata(): Promise<Metadata> {
  const { t, locale } = await getServerI18n();
  return pageMetadata({ title: t('about.title'), description: t('about.lead'), path: '/about', locale });
}

/** Tiêu đề ở cột trái, nội dung ở 2/3 bên phải — dòng chữ không dài quá ~70 ký tự trên màn rộng. */
const split = 'grid gap-4 lg:grid-cols-3 lg:gap-10';
const heading = 'text-2xl font-bold text-ink';
const prose = 'max-w-prose text-base leading-relaxed text-muted';
const iconTile = 'inline-flex h-11 w-11 shrink-0 items-center justify-center rounded-ui bg-primary-soft text-primary-ink';

export default async function AboutPage() {
  const { t } = await getServerI18n();
  const subjects = [
    { key: 'development', icon: Braces },
    { key: 'data', icon: Database },
    { key: 'delivery', icon: Workflow },
  ] as const;
  const perspectives = ['requirements', 'code', 'data', 'testing', 'deploy'] as const;
  const starts = [
    { icon: Route, title: t('about.guidedTitle'), body: t('about.guidedBody'), href: '/learning-paths', label: t('home.ctaStart'), button: btnPrimary },
    { icon: BookOpen, title: t('about.browseTitle'), body: t('about.browseBody'), href: '/articles', label: t('home.ctaArticles'), button: btnSecondary },
  ];

  return (
    <div className={container}>
      <PageBanner variant="about" title={t('about.title')} subtitle={t('about.lead')} />
      <div className="space-y-12 pb-12 lg:space-y-16 lg:pb-16">
        <section aria-labelledby="about-audience" className={split}>
          <h2 id="about-audience" className={heading}>{t('about.audienceTitle')}</h2>
          <p className={`${prose} lg:col-span-2`}>{t('about.audience')}</p>
        </section>

        <section aria-labelledby="about-content" className={split}>
          <h2 id="about-content" className={heading}>{t('about.whatTitle')}</h2>
          <div className="space-y-5 lg:col-span-2">
            <p className={prose}>{t('about.what')}</p>
            <ul className="space-y-3">
              {subjects.map(({ key, icon: Icon }) => <li key={key} className={`${card} flex gap-4 p-4 sm:p-5`}>
                <span className={iconTile}><Icon aria-hidden="true" className="h-5 w-5" /></span>
                <div className="min-w-0">
                  <h3 className="text-lg font-semibold text-ink">{t(`about.subjects.${key}.title`)}</h3>
                  <p className="mt-1 text-base leading-relaxed text-muted">{t(`about.subjects.${key}.body`)}</p>
                </div>
              </li>)}
            </ul>
          </div>
        </section>

        <section aria-labelledby="about-project" className="grid gap-8 rounded-card border border-line bg-surface-2 p-5 sm:p-8 lg:grid-cols-3 lg:gap-10">
          <div className="space-y-4 lg:col-span-2">
            <span className={iconTile}><MapPin aria-hidden="true" className="h-5 w-5" /></span>
            <h2 id="about-project" className={heading}>{t('about.projectTitle')}</h2>
            <p className={prose}>{t('about.projectBody')}</p>
            <Link href="/articles/du-an-mau-toi-noi" className={`${btnLink} min-h-11`}>
              {t('home.projectCta')}<ArrowRight aria-hidden="true" className="h-4 w-4 shrink-0" />
            </Link>
          </div>
          <div>
            <h3 className="text-sm font-semibold text-ink">{t('about.perspectivesTitle')}</h3>
            <ol className="mt-4 space-y-3 border-l border-line pl-5">
              {perspectives.map((key, i) => <li key={key} className="relative flex items-baseline gap-3 text-base text-ink">
                <span aria-hidden="true" className="absolute top-2 -left-[25px] h-2.5 w-2.5 rounded-full border-2 border-primary-ink bg-surface-2" />
                <span aria-hidden="true" className="font-mono text-sm text-muted">{String(i + 1).padStart(2, '0')}</span>
                {t(`about.perspectives.${key}`)}
              </li>)}
            </ol>
          </div>
        </section>

        <section aria-labelledby="about-start" className={split}>
          <h2 id="about-start" className={heading}>{t('about.howTitle')}</h2>
          <div className="space-y-5 lg:col-span-2">
            <div className="grid gap-4 sm:grid-cols-2">
              {starts.map(({ icon: Icon, title, body, href, label, button }) => <div key={href} className={`${card} flex flex-col items-start gap-3 p-5 sm:p-6`}>
                <span className={iconTile}><Icon aria-hidden="true" className="h-5 w-5" /></span>
                <h3 className="text-lg font-semibold text-ink">{title}</h3>
                <p className="text-base leading-relaxed text-muted">{body}</p>
                <Link href={href} className={`${button} mt-auto min-h-11 w-full`}>{label}</Link>
              </div>)}
            </div>
            <p className="flex max-w-prose gap-2 text-sm leading-relaxed text-muted">
              <Info aria-hidden="true" className="mt-0.5 h-4 w-4 shrink-0" />{t('about.progressNote')}
            </p>
          </div>
        </section>

        <section aria-labelledby="about-discussion" className={`${split} border-t border-line pt-10 lg:pt-12`}>
          <h2 id="about-discussion" className={heading}>{t('about.joinTitle')}</h2>
          <div className="space-y-3 lg:col-span-2">
            <p className={prose}>{t('about.join')}</p>
            <Link href="/articles" className={`${btnLink} min-h-11`}>
              {t('about.discussionCta')}<ArrowRight aria-hidden="true" className="h-4 w-4 shrink-0" />
            </Link>
          </div>
        </section>
      </div>
    </div>
  );
}
