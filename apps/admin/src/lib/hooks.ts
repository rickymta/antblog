import { useCallback, useEffect, useState } from 'react';
import { useSearchParams } from 'react-router';

export function useDebounced<T>(value: T, delay = 300): T {
  const [v, setV] = useState(value);
  useEffect(() => {
    const id = window.setTimeout(() => setV(value), delay);
    return () => window.clearTimeout(id);
  }, [value, delay]);
  return v;
}

/**
 * Bộ lọc danh sách lưu trên URL (giữ khi tải lại/quay lại). Đổi một bộ lọc khác `page` ⇒ về trang 1.
 * Giá trị rỗng bị bỏ khỏi URL.
 */
export function useUrlFilters<K extends string>(keys: readonly K[]) {
  const [params, setParams] = useSearchParams();
  const values = Object.fromEntries(keys.map((k) => [k, params.get(k) ?? ''])) as Record<K, string>;
  const pageRaw = Number(params.get('page') ?? '1');
  const page = Number.isInteger(pageRaw) && pageRaw > 0 ? pageRaw : 1;

  /** Đặt nhiều khoá trong MỘT lần ghi URL (gọi setParams hai lần liền nhau thì lần sau đè lần trước). */
  const setMany = useCallback(
    (entries: Partial<Record<K | 'page', string>>) => {
      setParams(
        (prev) => {
          const next = new URLSearchParams(prev);
          let resetPage = false;
          for (const [key, value] of Object.entries(entries) as [string, string | undefined][]) {
            if (value) next.set(key, value);
            else next.delete(key);
            if (key !== 'page') resetPage = true;
          }
          if (resetPage && !('page' in entries)) next.delete('page');
          return next;
        },
        { replace: true },
      );
    },
    [setParams],
  );

  const set = useCallback(
    (key: K | 'page', value: string) => setMany({ [key]: value } as Partial<Record<K | 'page', string>>),
    [setMany],
  );

  const clear = useCallback(() => setParams(new URLSearchParams(), { replace: true }), [setParams]);
  const setPage = useCallback((p: number) => set('page', p > 1 ? String(p) : ''), [set]);
  const active = keys.some((k) => values[k] !== '');

  return { values, page, set, setMany, setPage, clear, active };
}

/** Ô tìm kiếm gõ tự do, đẩy lên URL sau khi ngừng gõ. */
export function useSearchBox(urlValue: string, commit: (v: string) => void) {
  const [text, setText] = useState(urlValue);
  const debounced = useDebounced(text.trim(), 350);
  useEffect(() => {
    if (debounced !== urlValue) commit(debounced);
    // eslint-disable-next-line react-hooks/exhaustive-deps -- chỉ đẩy khi giá trị đã gõ ổn định
  }, [debounced]);
  useEffect(() => {
    // URL bị xoá lọc từ ngoài ⇒ đồng bộ lại ô nhập.
    if (urlValue === '' && debounced !== '') setText('');
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [urlValue]);
  return [text, setText] as const;
}
