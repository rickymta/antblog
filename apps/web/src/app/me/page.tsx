import type { Metadata } from 'next';
import { getServerI18n } from '../../i18n/server';
import { ProfileView } from '../../views/ProfileView';

export async function generateMetadata(): Promise<Metadata> {
  const { t } = await getServerI18n();
  return { title: t('profile.title'), robots: { index: false, follow: false } };
}

export default function ProfilePage() {
  return <ProfileView />;
}
