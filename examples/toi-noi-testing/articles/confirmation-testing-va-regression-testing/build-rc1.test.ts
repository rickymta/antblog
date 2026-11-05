/**
 * Build rc1 — bản có defect làm tròn (toFixed). Đây là lần chạy ĐẦU TIÊN tester ghi nhận failure.
 * Ca CF-05, CF-06 chưa tồn tại lúc đó; chạy ở đây để thấy defect cũng làm sai khối lượng.
 */
import { vi } from 'vitest';
import { defineSuite } from './define-suite';

vi.mock('../../src/domain/money', async (importOriginal) => {
  const original = await importOriginal<typeof import('../../src/domain/money')>();
  const { roundTenthRc1 } = await import('./rounding-variants');
  return { ...original, roundToTenth: roundTenthRc1 };
});

defineSuite('rc1 (toFixed — có defect)', new Set(['CF-01', 'CF-02', 'CF-03', 'CF-04', 'CF-05', 'CF-06']));
