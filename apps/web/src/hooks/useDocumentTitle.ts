import { useEffect } from 'react';
import { useTranslation } from 'react-i18next';

/** Đặt `<title>` dạng "Tiêu đề · Tech Knowledge Hub"; không truyền gì ⇒ chỉ tên site. */
export function useDocumentTitle(title?: string): void {
  const { t } = useTranslation();
  const suffix = t('meta.titleSuffix');
  useEffect(() => {
    document.title = title ? `${title} · ${suffix}` : suffix;
  }, [title, suffix]);
}
