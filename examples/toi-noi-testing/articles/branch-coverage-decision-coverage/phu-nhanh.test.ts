/**
 * TEST-044 · Phủ nhánh src/domain/cancel.ts trên mã THẬT của lab.
 * Đo từng bước bằng bộ lọc tên ca (-t), xem README.md.
 */
import { describe } from 'vitest';
import { customerCancellationFee } from '../../src/domain/cancel';
import { CA_HUY, dangKy } from './ca-huy';

describe('cancel.ts (mã lab)', () => {
  dangKy(customerCancellationFee, CA_HUY.map((c) => c.id));
});
