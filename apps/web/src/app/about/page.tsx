import type { Metadata } from 'next';
import Link from 'next/link';
import { BookOpen, MessageSquare, Route } from 'lucide-react';
import { PageHeader } from '../../components/common/PageHeader';
import { getServerI18n } from '../../i18n/server';
import { pageMetadata } from '../../lib/metadata';
import { btnPrimary, btnSecondary, card, container } from '../../lib/ui';

export async function generateMetadata(): Promise<Metadata> {
  const { t, locale } = await getServerI18n();
  return pageMetadata({ title: t('about.title'), description: t('about.lead'), path: '/about', locale });
}

export default async function AboutPage() {
  const { t } = await getServerI18n();
  const blocks = [
    { icon: BookOpen, title: t('about.whatTitle'), body: t('about.what') },
    { icon: Route, title: t('about.howTitle'), body: t('about.how') },
    { icon: MessageSquare, title: t('about.joinTitle'), body: t('about.join') },
  ];
  return (
    <div className={`${container} max-w-4xl`}>
      <PageHeader title={t('about.title')} subtitle={t('about.lead')} />
      <div className="grid gap-5 md:grid-cols-3">
        {blocks.map(({ icon: Icon, title, body }) => (
          <section key={title} className={`p-5 ${card}`}>
            <Icon aria-hidden="true" className="h-6 w-6 text-primary" />
            <h2 className="mt-3 font-bold text-ink">{title}</h2>
            <p className="mt-2 text-sm leading-relaxed text-muted">{body}</p>
          </section>
        ))}
      </div>
      <div className="mt-8 flex flex-wrap gap-3">
        <Link href="/learning-paths" className={btnPrimary}>
          {t('home.ctaStart')}
        </Link>
        <Link href="/articles" className={btnSecondary}>
          {t('home.ctaArticles')}
        </Link>
      </div>
    </div>
  );
}
