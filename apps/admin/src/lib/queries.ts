import { useQuery } from '@tanstack/react-query';
import { api } from '@tkh/shared';

/** Danh sách chủ đề cho ô chọn — dùng API công khai vì tác giả không gọi được /admin/categories. */
export function useCategoryOptions() {
  return useQuery({ queryKey: ['categories', 'options'], queryFn: () => api.categories.list(), staleTime: 5 * 60_000 });
}

/** Từ điển keyword cho bản xem trước bài viết — API công khai, nên tác giả cũng gọi được. */
export function useGlossary(language: 'vi' | 'en') {
  return useQuery({ queryKey: ['glossary', language], queryFn: () => api.glossary.list(language), staleTime: 5 * 60_000 });
}
