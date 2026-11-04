// Bản sao lưu cục bộ của trình soạn bài (localStorage): ghi vài giây một lần khi có thay đổi chưa lưu,
// đề nghị khôi phục khi mở lại nếu mới hơn bản trên máy chủ. Storage bị chặn ⇒ mọi hàm im lặng.

const PREFIX = 'tkh-admin-draft:';

export interface DraftBackup<T> {
  form: T;
  /** Mốc ms lúc ghi bản sao. */
  savedAt: number;
}

export function draftKey(articleId: string | undefined): string {
  return `${PREFIX}${articleId ?? 'new'}`;
}

export function readDraft<T>(key: string): DraftBackup<T> | undefined {
  try {
    const raw = localStorage.getItem(key);
    if (!raw) return undefined;
    const v = JSON.parse(raw) as DraftBackup<T>;
    return v && typeof v.savedAt === 'number' && v.form ? v : undefined;
  } catch {
    return undefined;
  }
}

export function writeDraft<T>(key: string, form: T): number | undefined {
  const savedAt = Date.now();
  try {
    localStorage.setItem(key, JSON.stringify({ form, savedAt } satisfies DraftBackup<T>));
    return savedAt;
  } catch {
    return undefined;
  }
}

export function removeDraft(key: string): void {
  try {
    localStorage.removeItem(key);
  } catch {
    /* bỏ qua */
  }
}
