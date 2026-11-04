import { useEffect } from 'react';
import { usePathname } from 'next/navigation';
import { api } from '@tkh/shared';
import { readStorage, writeStorage } from '../lib/storage';

const FLAG = 'tkh-visit-tracked';
// Dự phòng khi sessionStorage bị chặn: ít nhất không gửi lại trong cùng lần tải trang.
let trackedInMemory = false;

/** Ghi một lượt truy cập MỘT lần mỗi phiên trình duyệt (cờ ở sessionStorage), với đường dẫn trang đầu tiên. */
export function useVisitTracking(): void {
  const pathname = usePathname();
  useEffect(() => {
    if (trackedInMemory || readStorage(FLAG, 'session')) return;
    trackedInMemory = true;
    writeStorage(FLAG, '1', 'session');
    api.site.trackVisit(pathname).catch(() => {
      /* thống kê không được làm hỏng trang */
    });
    // Chỉ chạy lúc vào trang lần đầu — các lượt điều hướng sau không phải lượt truy cập mới.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);
}
