import { useTranslation } from 'react-i18next';
import { CommunitySection } from '../components/home/CommunitySection';
import { Hero } from '../components/home/Hero';
import { LatestSection, PathsSection, TopicTiles, TrendingSection } from '../components/home/Sections';
import { useDocumentTitle } from '../hooks/useDocumentTitle';

export function HomePage() {
  const { t } = useTranslation();
  useDocumentTitle(t('home.title'));
  return (
    <>
      <Hero />
      <TopicTiles />
      <TrendingSection />
      <PathsSection />
      <LatestSection />
      <CommunitySection />
    </>
  );
}
