import type { Metadata } from 'next';
import { pickLocalized } from '@tkh/shared';
import { I18N_ENABLED } from '../../../i18n/config';
import { getServerI18n } from '../../../i18n/server';
import { breadcrumbList, JsonLd } from '../../../lib/jsonld';
import { clampDescription, notFoundMetadata, pageMetadata } from '../../../lib/metadata';
import { orNotFound, orNull, serverApi } from '../../../lib/server-api';
import { absoluteUrl, SITE_URL } from '../../../lib/site';
import { PathDetailView } from '../../../views/PathDetailView';

type Props = { params: Promise<{ slug: string }> };

/**
 * ISR: dựng trang lộ trình lần đầu khi có request rồi cache ở máy chủ `revalidate` giây (HTML không phụ thuộc cookie/request).
 * generateStaticParams trả [] ⇒ KHÔNG dựng gì lúc `next build` (ảnh Docker build không có API); slug nào cũng hợp lệ
 * (dynamicParams mặc định true). Slug không tồn tại bị Proxy chặn trước (src/proxy.ts).
 * Bật song ngữ (NEXT_PUBLIC_ENABLE_I18N) ⇒ máy chủ đọc cookie ngôn ngữ ⇒ trang phải động: bỏ generateStaticParams.
 */
export const revalidate = 60;
export const generateStaticParams = I18N_ENABLED ? undefined : async (): Promise<{ slug: string }[]> => [];

const pathHref = (slug: string) => `/learning-paths/${encodeURIComponent(slug)}`;

export async function generateMetadata({ params }: Props): Promise<Metadata> {
  const { slug } = await params;
  const { t, locale } = await getServerI18n();
  const path = await orNull(serverApi.learningPath(slug));
  if (!path) return notFoundMetadata();
  return pageMetadata({
    title: path.title,
    description: clampDescription(path.description || t('paths.subtitle')),
    path: pathHref(path.slug),
    locale,
  });
}

export default async function LearningPathPage({ params }: Props) {
  const { slug } = await params;
  const { t, locale } = await getServerI18n();
  const path = await orNotFound(serverApi.learningPath(slug));
  const url = absoluteUrl(pathHref(path.slug));

  const itemList = {
    '@context': 'https://schema.org',
    '@type': 'ItemList',
    name: path.title,
    description: path.description,
    url,
    numberOfItems: path.articles.length,
    itemListOrder: 'https://schema.org/ItemListOrderAscending',
    itemListElement: path.articles.map((a, i) => ({ '@type': 'ListItem', position: i + 1, url: absoluteUrl(`/articles/${encodeURIComponent(a.slug)}`), name: a.title })),
  };
  const crumbs = [
    { name: t('nav.home'), url: `${SITE_URL}/` },
    { name: t('nav.paths'), url: absoluteUrl('/learning-paths') },
    ...(path.category ? [{ name: pickLocalized(path.category.name, locale), url: absoluteUrl(`/topics/${encodeURIComponent(path.category.slug)}`) }] : []),
    { name: path.title, url },
  ];

  return (
    <>
      <JsonLd data={breadcrumbList(crumbs)} />
      <JsonLd data={itemList} />
      <PathDetailView path={path} />
    </>
  );
}
