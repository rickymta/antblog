// Truy cập localStorage/sessionStorage an toàn: chế độ riêng tư hoặc trình duyệt chặn storage sẽ ném lỗi — nuốt lỗi, trả mặc định.

type Area = 'local' | 'session';

function area(kind: Area): Storage | null {
  try {
    return kind === 'local' ? window.localStorage : window.sessionStorage;
  } catch {
    return null;
  }
}

export function readStorage(key: string, kind: Area = 'local'): string | null {
  try {
    return area(kind)?.getItem(key) ?? null;
  } catch {
    return null;
  }
}

/** Trả `false` nếu không ghi được (storage bị chặn hoặc đầy). */
export function writeStorage(key: string, value: string, kind: Area = 'local'): boolean {
  try {
    const s = area(kind);
    if (!s) return false;
    s.setItem(key, value);
    return true;
  } catch {
    return false;
  }
}

export function readJson<T>(key: string, isValid: (v: unknown) => v is T, fallback: T): T {
  const raw = readStorage(key);
  if (!raw) return fallback;
  try {
    const parsed: unknown = JSON.parse(raw);
    return isValid(parsed) ? parsed : fallback;
  } catch {
    return fallback;
  }
}
