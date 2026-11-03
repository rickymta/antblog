import { lazy, Suspense } from 'react';
import { createBrowserRouter, RouterProvider } from 'react-router';
import { PageLoading } from './components/common/States';
import { Layout } from './components/layout/Layout';
import { AboutPage } from './pages/AboutPage';
import { ArticlesPage } from './pages/ArticlesPage';
import { LoginPage, RegisterPage } from './pages/AuthPages';
import { HomePage } from './pages/HomePage';
import { LearningPathPage } from './pages/LearningPathPage';
import { LearningPathsPage } from './pages/LearningPathsPage';
import { NotFoundPage } from './pages/NotFoundPage';
import { ProfilePage } from './pages/ProfilePage';
import { RouteError } from './pages/RouteError';
import { TopicPage } from './pages/TopicPage';

// Trang bài viết kéo theo bộ hiển thị Markdown + highlight.js (phần nặng nhất) ⇒ tách gói, các trang khác tải nhanh hơn.
const ArticlePage = lazy(() => import('./pages/ArticlePage').then((m) => ({ default: m.ArticlePage })));

const router = createBrowserRouter([
  {
    element: <Layout />,
    errorElement: <RouteError />,
    children: [
      { index: true, element: <HomePage /> },
      { path: 'articles', element: <ArticlesPage /> },
      {
        path: 'articles/:slug',
        element: (
          <Suspense fallback={<PageLoading />}>
            <ArticlePage />
          </Suspense>
        ),
      },
      { path: 'topics/:slug', element: <TopicPage /> },
      { path: 'learning-paths', element: <LearningPathsPage /> },
      { path: 'learning-paths/:slug', element: <LearningPathPage /> },
      { path: 'about', element: <AboutPage /> },
      { path: 'login', element: <LoginPage /> },
      { path: 'register', element: <RegisterPage /> },
      { path: 'me', element: <ProfilePage /> },
      { path: '*', element: <NotFoundPage /> },
    ],
  },
]);

export function App() {
  return <RouterProvider router={router} />;
}
