import { randomUUID } from 'node:crypto';
import type { IdGenerator } from '../app/ports';

/** UUID ngẫu nhiên — dùng khi chạy thật. */
export class RandomIdGenerator implements IdGenerator {
  next(): string {
    return randomUUID();
  }
}

/** Id tuần tự dễ đọc (`id-0001`, `id-0002`…) — dùng trong test để kết quả tất định. */
export class SequentialIdGenerator implements IdGenerator {
  private counter = 0;

  constructor(private readonly prefix = 'id-') {}

  next(): string {
    this.counter += 1;
    return `${this.prefix}${String(this.counter).padStart(4, '0')}`;
  }
}
