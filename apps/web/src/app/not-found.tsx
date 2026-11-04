import type { Metadata } from 'next';
import { NotFoundView } from '../views/NotFoundView';
import { getServerI18n } from '../i18n/server';

export async function generateMetadata(): Promise<Metadata> {
  const { t } = await getServerI18n();
  return { title: t('notFound.title'), robots: { index: false, follow: true } };
}

export default function NotFound() {
  return <NotFoundView />;
}
