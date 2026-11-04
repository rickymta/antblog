import type { Metadata } from 'next';
import { NotFoundView } from '../views/NotFoundView';
import { getServerI18n } from '../i18n/server';

export async function generateMetadata(): Promise<Metadata> {
  const { t } = await getServerI18n();
  return { title: t('notFound.title'), robots: { index: false, follow: true } };
}

/**
 * Trang 404 gốc — render NGAY trên máy chủ (mã 404). Bài / chủ đề / lộ trình không tồn tại được Proxy rewrite về đây
 * (src/proxy.ts); NotFoundView tự chọn câu báo riêng theo URL đang mở.
 * Lưu ý: KHÔNG gọi headers()/cookies() ở đây — Next dựng sẵn phần tử not-found gốc cho MỌI trang (làm ranh giới 404),
 * gọi API động là biến cả trang ISR thành động.
 */
export default function NotFound() {
  return <NotFoundView />;
}
