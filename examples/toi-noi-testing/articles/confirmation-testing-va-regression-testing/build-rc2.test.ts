/**
 * Build rc2 — «sửa» lần 1 bằng Math.ceil. Confirmation test đạt hết, regression test bắt được
 * tác dụng phụ ở biên quãng đường, biên khối lượng và phí giao đồ ăn.
 */
import { vi } from 'vitest';
import { defineSuite } from './define-suite';

vi.mock('../../src/domain/money', async (importOriginal) => {
  const original = await importOriginal<typeof import('../../src/domain/money')>();
  const { roundTenthRc2 } = await import('./rounding-variants');
  return { ...original, roundToTenth: roundTenthRc2 };
});

defineSuite('rc2 (Math.ceil — sửa lần 1)', new Set(['RG-01', 'RG-03', 'RG-05', 'RG-06', 'RG-08', 'RG-09']));
