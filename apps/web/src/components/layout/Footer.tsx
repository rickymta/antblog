import type { ReactNode } from 'react';
import { Link } from 'react-router';
import { useTranslation } from 'react-i18next';
import { useQuery } from '@tanstack/react-query';
import { pickLocalized } from '@tkh/shared';
import { useLocale } from '../../i18n';
import { useAuth } from '../../lib/auth';
import { container } from '../../lib/ui';
import { LogoMark } from './Logo';
import { categoriesQuery } from './TopicsMenu';

const linkCls = 'text-sm text-muted hover:text-ink hover:underline underline-offset-4';

function Column({ title, children }: { title: string; children: ReactNode }) {
  return (
    <div>
      <h2 className="mb-3 text-sm font-bold text-ink">{title}</h2>
      <ul className="space-y-2">{children}</ul>
    </div>
  );
}

/** Chân trang 4 cột: giới thiệu · chủ đề · tài nguyên · kết nối + dòng bản quyền. */
export function Footer() {
  const { t } = useTranslation();
  const locale = useLocale();
  const { user } = useAuth();
  const categories = useQuery(categoriesQuery);

  return (
    <footer className="mt-16 border-t border-line bg-surface">
      <div className={`${container} grid gap-10 py-12 sm:grid-cols-2 lg:grid-cols-4`}>
        <div className="space-y-3">
          <p className="flex items-center gap-2 font-bold text-ink">
            <LogoMark className="h-8 w-8 text-xs" />
            {t('meta.siteName')}
          </p>
          <p className="text-sm text-muted">{t('footer.about')}</p>
        </div>
        <Column title={t('footer.topics')}>
          {(categories.data ?? []).slice(0, 6).map((c) => (
            <li key={c.id}>
              <Link to={`/topics/${encodeURIComponent(c.slug)}`} className={linkCls}>
                {pickLocalized(c.name, locale)}
              </Link>
            </li>
          ))}
          <li>
            <Link to="/articles" className={linkCls}>
              {t('nav.allTopics')}
            </Link>
          </li>
        </Column>
        <Column title={t('footer.resources')}>
          <li>
            <Link to="/articles" className={linkCls}>
              {t('footer.allArticles')}
            </Link>
          </li>
          <li>
            <Link to="/articles?sort=trending" className={linkCls}>
              {t('footer.trending')}
            </Link>
          </li>
          <li>
            <Link to="/learning-paths" className={linkCls}>
              {t('footer.learningPaths')}
            </Link>
          </li>
          <li>
            <Link to="/about" className={linkCls}>
              {t('nav.about')}
            </Link>
          </li>
        </Column>
        <Column title={t('footer.connect')}>
          <li>
            <Link to="/#newsletter" className={linkCls}>
              {t('footer.newsletter')}
            </Link>
          </li>
          <li>
            {user ? (
              <Link to="/me" className={linkCls}>
                {t('footer.account')}
              </Link>
            ) : (
              <Link to="/register" className={linkCls}>
                {t('footer.createAccount')}
              </Link>
            )}
          </li>
        </Column>
      </div>
      <div className="border-t border-line">
        <p className={`${container} py-5 text-xs text-muted`}>{t('footer.copyright', { year: new Date().getFullYear() })}</p>
      </div>
    </footer>
  );
}
