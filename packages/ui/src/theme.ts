'use client';

import { useCallback, useEffect, useState } from 'react';

export type ThemeMode = 'light' | 'dark';
const KEY = 'tkh-theme';

function current(): ThemeMode {
  return document.documentElement.classList.contains('dark') ? 'dark' : 'light';
}

/**
 * Sáng/tối dùng chung hai app. index.html đã áp class `.dark` trước khi React chạy;
 * hook này chỉ đọc trạng thái đó, đổi và nhớ vào localStorage (lỗi storage thì vẫn đổi được trong phiên).
 */
export function useTheme(): { theme: ThemeMode; toggle: () => void; setTheme: (t: ThemeMode) => void } {
  const [theme, setThemeState] = useState<ThemeMode>(() => (typeof document === 'undefined' ? 'light' : current()));

  const setTheme = useCallback((t: ThemeMode) => {
    document.documentElement.classList.toggle('dark', t === 'dark');
    try {
      localStorage.setItem(KEY, t);
    } catch {
      /* chế độ riêng tư / chặn storage: bỏ qua */
    }
    setThemeState(t);
  }, []);

  useEffect(() => {
    // Người dùng chưa chọn tay ⇒ theo hệ điều hành khi hệ điều hành đổi.
    const mq = window.matchMedia('(prefers-color-scheme: dark)');
    const onChange = (e: MediaQueryListEvent) => {
      let saved: string | null = null;
      try {
        saved = localStorage.getItem(KEY);
      } catch {
        /* bỏ qua */
      }
      if (!saved) {
        document.documentElement.classList.toggle('dark', e.matches);
        setThemeState(e.matches ? 'dark' : 'light');
      }
    };
    mq.addEventListener('change', onChange);
    return () => mq.removeEventListener('change', onChange);
  }, []);

  const toggle = useCallback(() => setTheme(current() === 'dark' ? 'light' : 'dark'), [setTheme]);
  return { theme, toggle, setTheme };
}
