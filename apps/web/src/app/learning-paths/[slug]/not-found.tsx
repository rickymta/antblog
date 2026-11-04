import { NotFoundView } from '../../../views/NotFoundView';

// Dự phòng: slug không tồn tại thường bị Proxy (src/proxy.ts) rewrite về trang 404 gốc render sẵn trên máy chủ. Tới được
// đây (Proxy không hỏi được API, hoặc bài vừa bị gỡ) thì mã 404 + noindex vẫn đúng nhưng nội dung dựng ở trình duyệt.
export default function LearningPathNotFound() {
  return <NotFoundView messageKey="paths.notFound" />;
}
