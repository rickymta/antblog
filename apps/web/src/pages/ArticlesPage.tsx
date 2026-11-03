import { useTranslation } from 'react-i18next';
import { ArticleBrowser } from '../components/common/ArticleBrowser';
import { PageHeader } from '../components/common/PageHeader';
import { useDocumentTitle } from '../hooks/useDocumentTitle';
import { container } from '../lib/ui';

export function ArticlesPage() {
  const { t } = useTranslation();
  useDocumentTitle(t('articles.title'));
  return (
    <div className={container}>
      <PageHeader title={t('articles.title')} subtitle={t('articles.subtitle')} />
      <ArticleBrowser />
    </div>
  );
}
