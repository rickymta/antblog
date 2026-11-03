import { useEffect, useState } from 'react';

/**
 * Trả id tiêu đề đang đọc: tiêu đề cuối cùng đã đi qua mốc `offset` px tính từ đỉnh cửa sổ
 * (bù thanh đầu dính). Chưa tới tiêu đề nào ⇒ tiêu đề đầu tiên.
 */
export function useScrollSpy(ids: readonly string[], offset = 120): string | undefined {
  const [active, setActive] = useState<string | undefined>(ids[0]);
  const key = ids.join('|');

  useEffect(() => {
    if (ids.length === 0) {
      setActive(undefined);
      return;
    }
    let frame = 0;
    const compute = () => {
      frame = 0;
      let current = ids[0];
      for (const id of ids) {
        const el = document.getElementById(id);
        if (!el) continue;
        if (el.getBoundingClientRect().top - offset <= 0) current = id;
        else break;
      }
      // Cuộn chạm đáy trang ⇒ tiêu đề cuối (đoạn cuối ngắn không bao giờ chạm mốc).
      if (window.innerHeight + window.scrollY >= document.documentElement.scrollHeight - 2) {
        current = ids[ids.length - 1];
      }
      setActive(current);
    };
    const onScroll = () => {
      if (!frame) frame = window.requestAnimationFrame(compute);
    };
    compute();
    window.addEventListener('scroll', onScroll, { passive: true });
    window.addEventListener('resize', onScroll);
    return () => {
      window.removeEventListener('scroll', onScroll);
      window.removeEventListener('resize', onScroll);
      if (frame) window.cancelAnimationFrame(frame);
    };
    // `key` thay cho mảng ids để không đăng ký lại khi mảng mới cùng nội dung.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [key, offset]);

  return active;
}
