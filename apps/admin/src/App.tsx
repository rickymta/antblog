import { lazy, Suspense, useState, type ReactNode } from 'react';
import { Link, RouterProvider, createBrowserRouter, useRouteError } from 'react-router';
import { QueryClientProvider } from '@tanstack/react-query';
import { useTranslation } from 'react-i18next';
import { createQueryClient } from './lib/session';
import { ToastProvider } from './lib/toast';
import { AdminOnly, SessionGate } from './layout/AppShell';
import { Card, ErrorState, Spinner, buttonClass } from './components/ui';
import { ArticlesPage } from './pages/ArticlesPage';
import { CategoriesPage } from './pages/CategoriesPage';
import { LearningPathsPage } from './pages/LearningPathsPage';
import { LearningPathEditorPage } from './pages/LearningPathEditorPage';
import { CommentsPage } from './pages/CommentsPage';
import { UsersPage } from './pages/UsersPage';
import { SubscribersPage } from './pages/SubscribersPage';

// Hai trang nặng (biểu đồ recharts; bộ hiển thị Markdown + highlight.js) tách thành chunk riêng.
const DashboardPage = lazy(() => import('./pages/DashboardPage').then((m) => ({ default: m.DashboardPage })));
const ArticleEditorPage = lazy(() => import('./pages/ArticleEditorPage').then((m) => ({ default: m.ArticleEditorPage })));

function Lazy({ children }: { children: ReactNode }) {
  const { t } = useTranslation();
  return <Suspense fallback={<Spinner label={t('common.loading')} />}>{children}</Suspense>;
}

function NotFound() {
  const { t } = useTranslation();
  return (
    <Card>
      <ErrorState title={t('common.notFoundTitle')} message={t('common.notFoundBody')} />
      <div className="flex justify-center pb-10">
        <Link to="/" className={buttonClass('primary')}>
          {t('common.goHome')}
        </Link>
      </div>
    </Card>
  );
}

/** Lỗi bất ngờ khi vẽ một trang ⇒ khối lỗi thay vì màn trắng. */
function RouteError({ inline }: { inline?: boolean }) {
  const { t } = useTranslation();
  const err = useRouteError();
  const message = err instanceof Error ? err.message : t('errors.unknown');
  if (inline) {
    return (
      <Card>
        <ErrorState message={message} onRetry={() => window.location.reload()} />
      </Card>
    );
  }
  return (
    <div className="flex min-h-dvh items-center justify-center p-4">
      <div className="w-full max-w-lg rounded-card border border-line bg-surface">
        <ErrorState message={message} onRetry={() => window.location.reload()} />
      </div>
    </div>
  );
}

const router = createBrowserRouter([
  {
    path: '/',
    element: <SessionGate />,
    errorElement: <RouteError />,
    children: [
      {
        errorElement: <RouteError inline />,
        children: [
          { index: true, element: <Lazy><DashboardPage /></Lazy> },
          { path: 'articles', element: <ArticlesPage /> },
          { path: 'articles/new', element: <Lazy><ArticleEditorPage /></Lazy> },
          { path: 'articles/:id', element: <Lazy><ArticleEditorPage /></Lazy> },
          { path: 'categories', element: <AdminOnly><CategoriesPage /></AdminOnly> },
          { path: 'learning-paths', element: <AdminOnly><LearningPathsPage /></AdminOnly> },
          { path: 'learning-paths/new', element: <AdminOnly><LearningPathEditorPage /></AdminOnly> },
          { path: 'learning-paths/:id', element: <AdminOnly><LearningPathEditorPage /></AdminOnly> },
          { path: 'comments', element: <AdminOnly><CommentsPage /></AdminOnly> },
          { path: 'users', element: <AdminOnly><UsersPage /></AdminOnly> },
          { path: 'subscribers', element: <AdminOnly><SubscribersPage /></AdminOnly> },
          { path: '*', element: <NotFound /> },
        ],
      },
    ],
  },
  // Dev chạy ở gốc cổng 5174 (BASE_URL '/'); bản Docker build với base '/admin/' sau nginx.
], { basename: import.meta.env.BASE_URL.replace(/\/+$/, '') || '/' });

export function App() {
  const [client] = useState(createQueryClient);
  return (
    <QueryClientProvider client={client}>
      <ToastProvider>
        <RouterProvider router={router} />
      </ToastProvider>
    </QueryClientProvider>
  );
}
