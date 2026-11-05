'use client';

import { useTransition } from 'react';
import Link from 'next/link';
import { useRouter } from 'next/navigation';
import { useTranslation } from 'react-i18next';
import { ArrowRight, BookOpen, RefreshCw } from 'lucide-react';
import { formatNumber, type SiteSummary } from '@tkh/shared';
import { useLocale } from '../../i18n';
import { container } from '../../lib/ui';
import { HeroWorkbench } from './HeroWorkbench';

export function Hero({ summary }: { summary: SiteSummary | null }) {
  const { t } = useTranslation();
  const locale = useLocale();
  const router = useRouter();
  const [refreshing, startTransition] = useTransition();
  const stats = [
    { value: summary?.articleCount, label: 'home.statArticles' },
    { value: summary?.learningPathCount, label: 'home.statPaths' },
    { value: summary?.categoryCount, label: 'home.statTopics' },
  ] as const;
  return (
    <section aria-labelledby="hero-title" className="home-hero">
      <div className={container + ' hero-layout'}>
        <div>
          <p className="editorial-eyebrow"><span className="h-2 w-2 rounded-full bg-primary" />{t('home.heroEyebrow')}</p>
          <h1 id="hero-title" className="hero-heading"><span>{t('home.heroLine1')}</span><span className="text-primary-ink">{t('home.heroLine2')}</span></h1>
          <p className="hero-description">{t('home.heroBody')}</p>
          <div className="mt-7 flex flex-wrap gap-3">
            <Link href="/learning-paths" className="hero-primary">{t('home.ctaStart')}<ArrowRight size={17} aria-hidden="true" /></Link>
            <Link href="/articles" className="hero-secondary"><BookOpen size={17} aria-hidden="true" />{t('home.ctaArticles')}</Link>
          </div>
          <div className="hero-stats" aria-busy={refreshing}>
            <dl aria-label={t('home.statsLabel')}>
              {stats.map(({ value, label }) => <div key={label}><dt>{t(label)}</dt><dd>{value === undefined ? '—' : formatNumber(value, locale)}</dd></div>)}
            </dl>
            {!summary && <button type="button" onClick={() => startTransition(() => router.refresh())} className="inline-flex items-center gap-2 text-sm text-primary-ink"><RefreshCw size={15} className={refreshing ? 'animate-spin' : ''} aria-hidden="true" />{t('common.retry')}</button>}
          </div>
        </div>
        <HeroWorkbench />
      </div>
    </section>
  );
}
