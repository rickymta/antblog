'use client';

import Link from 'next/link';
import { useTranslation } from 'react-i18next';
import { ArrowRight, BookOpen } from 'lucide-react';
import { container } from '../../lib/ui';

export function Hero() {
  const { t } = useTranslation();
  return (
    <section aria-labelledby="hero-title" className="home-hero">
      <div className={container + ' hero-layout'}>
        <div>
          <p className="editorial-eyebrow">{t('home.heroEyebrow')}</p>
          <h1 id="hero-title" className="hero-heading">{t('home.heroLine1')} <span>{t('home.heroLine2')}</span></h1>
          <p className="hero-description">{t('home.heroBody')}</p>
        </div>
        <nav aria-label={t('home.quickNav')} className="hero-actions">
          <Link href="/articles" className="hero-primary"><BookOpen size={18} aria-hidden="true" />{t('home.ctaArticles')}<ArrowRight size={17} aria-hidden="true" /></Link>
          <Link href="/learning-paths" className="hero-secondary">{t('home.ctaStart')}</Link>
        </nav>
      </div>
    </section>
  );
}
