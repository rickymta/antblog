import { useSyncExternalStore } from 'react';
import { readJson, writeStorage } from './storage';

/**
 * Kho nhỏ lưu JSON ở localStorage, đồng bộ giữa các tab (sự kiện `storage`) và giữa các component (useSyncExternalStore).
 * Snapshot giữ nguyên tham chiếu tới khi đổi ⇒ không render thừa. Không ghi được storage thì vẫn chạy trong bộ nhớ.
 */
export interface LocalStore<T> {
  get: () => T;
  set: (update: (prev: T) => T) => void;
  subscribe: (listener: () => void) => () => void;
}

export function createLocalStore<T>(key: string, isValid: (v: unknown) => v is T, fallback: T): LocalStore<T> {
  let cache: T | undefined;
  const listeners = new Set<() => void>();

  const get = (): T => {
    if (cache === undefined) cache = readJson(key, isValid, fallback);
    return cache;
  };
  const emit = () => listeners.forEach((l) => l());

  const onStorage = (e: StorageEvent) => {
    if (e.key !== key && e.key !== null) return;
    cache = undefined;
    emit();
  };

  return {
    get,
    set(update) {
      const next = update(get());
      if (next === cache) return;
      cache = next;
      writeStorage(key, JSON.stringify(next));
      emit();
    },
    subscribe(listener) {
      if (listeners.size === 0) window.addEventListener('storage', onStorage);
      listeners.add(listener);
      return () => {
        listeners.delete(listener);
        if (listeners.size === 0) window.removeEventListener('storage', onStorage);
      };
    },
  };
}

export function useLocalStore<T>(store: LocalStore<T>): T {
  return useSyncExternalStore(store.subscribe, store.get, store.get);
}

export const isRecord = (v: unknown): v is Record<string, unknown> =>
  typeof v === 'object' && v !== null && !Array.isArray(v);
