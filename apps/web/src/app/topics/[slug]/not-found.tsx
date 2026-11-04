import { NotFoundView } from '../../../views/NotFoundView';

// Mã 404 + noindex do Next đặt; khung HTML của trang 404 lồng nhau được trình duyệt dựng từ payload RSC (hành vi Next 16).
export default function TopicNotFound() {
  return <NotFoundView messageKey="topic.notFound" />;
}
