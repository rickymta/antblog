import type { Metadata } from 'next';
import { getServerI18n } from '../../i18n/server';
import { firstParam, type SearchParamsRecord } from '../../lib/articleQuery';
import { LoginView } from '../../views/AuthPages';

export async function generateMetadata(): Promise<Metadata> {
  const { t } = await getServerI18n();
  return { title: t('auth.loginTitle'), description: t('auth.loginSubtitle'), robots: { index: false, follow: false }, alternates: { canonical: '/login' } };
}

export default async function LoginPage({ searchParams }: { searchParams: Promise<SearchParamsRecord> }) {
  return <LoginView nextParam={firstParam((await searchParams).next)} />;
}
