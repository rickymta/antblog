import type { MetadataRoute } from 'next';
import { SITE_NAME } from '../lib/site';

export default function manifest(): MetadataRoute.Manifest {
  return {
    name: SITE_NAME,
    short_name: 'TKH',
    description: 'Blog kỹ thuật tiếng Việt về database, ngôn ngữ lập trình, design pattern và kiến trúc phần mềm.',
    start_url: '/',
    display: 'standalone',
    background_color: '#F8FAFC',
    theme_color: '#2563EB',
    lang: 'vi',
    icons: [
      { src: '/icon.svg', sizes: 'any', type: 'image/svg+xml' },
      { src: '/logo.png', sizes: '512x512', type: 'image/png', purpose: 'any' },
    ],
  };
}
