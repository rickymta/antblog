'use client';

import Link from 'next/link';
import { ArrowUpRight, Braces, Check, Cloud, Database, ListTree, MapPin, ShieldCheck } from 'lucide-react';
import { useTranslation } from 'react-i18next';

const TOPICS = [
  { key: 'programming', href: '/topics/programming', icon: Braces, detail: 'Java · Python · Go', tone: 'sky' },
  { key: 'testing', href: '/topics/testing', icon: ShieldCheck, detail: 'Manual · Automation', tone: 'green' },
  { key: 'ba', href: '/topics/business-analysis', icon: ListTree, detail: 'Requirements · BPMN', tone: 'amber' },
  { key: 'database', href: '/topics/database', icon: Database, detail: 'PostgreSQL · MongoDB', tone: 'violet' },
  { key: 'aws', href: '/topics/devops-aws', icon: Cloud, detail: 'AWS · CI/CD · Cloud', tone: 'amber' },
  { key: 'azure', href: '/topics/devops-azure', icon: Cloud, detail: 'Azure · CI/CD · Cloud', tone: 'sky' },
] as const;

export function HeroWorkbench() {
  const { t } = useTranslation();
  return (
    <nav className="workbench" aria-label={t('home.areasTitle')}>
      <div className="workbench-bar"><span className="flex gap-1.5" aria-hidden="true"><i /><i /><i /></span><span>toi-noi / knowledge-map</span><span aria-hidden="true">⌘</span></div>
      <div className="workbench-canvas">
        <p className="workbench-axis">{t('home.mapHint')}</p>
        <Link href="/articles/du-an-mau-toi-noi" className="workbench-project"><span className="workbench-pin"><MapPin size={25} aria-hidden="true" /></span><span><strong>Tới Nơi</strong><small>{t('home.projectType')}</small></span><ArrowUpRight size={16} aria-hidden="true" /></Link>
        <svg className="workbench-lines" viewBox="0 0 480 280" preserveAspectRatio="none" fill="none" aria-hidden="true"><path d="M240 30V70M80 110V70H400V110M240 70V110M80 220V110M240 220V110M400 220V110" stroke="currentColor" strokeWidth="1.5" strokeDasharray="5 5" /><circle cx="240" cy="70" r="4" fill="currentColor" /></svg>
        <ul className="workbench-nodes">{TOPICS.map(({key,href,icon:Icon,detail,tone}) => <li key={key}><Link href={href} className={'workbench-topic tone-' + tone}><Icon size={24} aria-hidden="true" /><strong>{key === 'aws' ? 'DevOps (AWS)' : key === 'azure' ? 'DevOps (Azure)' : t(`home.areas.${key}.title`)}</strong><small>{detail}</small><ArrowUpRight className="workbench-topic-arrow" size={12} aria-hidden="true" /></Link></li>)}</ul>
        <div className="workbench-terminal"><span>$ learn → build → share</span><span><Check size={14} aria-hidden="true" />{t('home.projectRunnable')}</span></div>
      </div>
      <div className="workbench-caption"><span>{t('home.projectCaption')}</span><span aria-hidden="true">↗</span></div>
    </nav>
  );
}
