/**
 * TEST-044 · Bộ ca chạy trên bản tái cấu trúc có lỗi cancel-v2.ts.
 * BC-01/02/03/05/06 phủ 100% câu lệnh của v2 và đều xanh; BC-07 (tài xế trễ) lộ defect —
 * khai `it.fails` để `npm test` vẫn xanh, đặt SHOW_RED=1 để thấy failure thật.
 */
import { describe, it } from 'vitest';
import { CA_HUY, dangKy } from './ca-huy';
import { customerCancellationFeeV2 } from './cancel-v2';

const itRed = process.env.SHOW_RED === '1' ? it : it.fails;
const caLoi = new Set(['BC-07']);

describe('cancel-v2.ts (bản tái cấu trúc có lỗi)', () => {
  dangKy(customerCancellationFeeV2, CA_HUY.map((c) => c.id), (id) => (caLoi.has(id) ? itRed : it));
});
