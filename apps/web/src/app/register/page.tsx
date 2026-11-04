import type { Metadata } from 'next';
import { getServerI18n } from '../../i18n/server';
import { firstParam, type SearchParamsRecord } from '../../lib/articleQuery';
import { RegisterView } from '../../views/AuthPages';

export async function generateMetadata(): Promise<Metadata> {
  const { t } = await getServerI18n();
  return { title: t('auth.registerTitle'), description: t('auth.registerSubtitle'), robots: { index: false, follow: false }, alternates: { canonical: '/register' } };
}

export default async function RegisterPage({ searchParams }: { searchParams: Promise<SearchParamsRecord> }) {
  return <RegisterView nextParam={firstParam((await searchParams).next)} />;
}
