import type { Customer, Driver } from '../domain/trip';
import type { Voucher } from '../domain/voucher';

/** Dữ liệu mẫu của máy chủ lab (SPEC Phụ lục A). Test có thể dùng lại hoặc tự dựng dữ liệu riêng. */
export interface SeedData {
  customers: Customer[];
  drivers: Driver[];
  vouchers: Voucher[];
}

export function labSeed(): SeedData {
  return {
    customers: [
      { id: 'C001', phone: '0900000001', fullName: 'Nguyễn Văn An' },
      { id: 'C002', phone: '0900000002', fullName: 'Trần Thị Bình' },
      { id: 'C003', phone: '0900000003', fullName: 'Lê Minh Châu' },
    ],
    drivers: [
      { id: 'D101', fullName: 'Phạm Văn Dũng', cityCode: 'SG', vehicle: 'bike', status: 'online' },
      { id: 'D102', fullName: 'Hoàng Thị Em', cityCode: 'SG', vehicle: 'bike', status: 'online' },
      { id: 'D103', fullName: 'Võ Quốc Phong', cityCode: 'SG', vehicle: 'car', status: 'online' },
      { id: 'D104', fullName: 'Đặng Văn Giang', cityCode: 'SG', vehicle: 'bike', status: 'offline' },
      { id: 'D201', fullName: 'Bùi Thị Hoa', cityCode: 'HN', vehicle: 'bike', status: 'online' },
      { id: 'D202', fullName: 'Đỗ Mạnh Hùng', cityCode: 'HN', vehicle: 'car', status: 'online' },
    ],
    vouchers: [
      voucher({ code: 'GIAM20', percent: 20, maxDiscount: 30_000, minAmount: 0 }),
      voucher({ code: 'FOOD15', percent: 15, maxDiscount: 25_000, minAmount: 50_000, kinds: ['food'], perCustomerPerDay: 2 }),
      voucher({ code: 'HN10', percent: 10, maxDiscount: 15_000, minAmount: 40_000, kinds: ['ride'], cityCodes: ['HN'] }),
      voucher({ code: 'XMAS50', percent: 50, maxDiscount: 50_000, minAmount: 0, validFrom: '2025-12-24', validTo: '2025-12-26' }),
    ],
  };
}

/** Dựng voucher với giá trị mặc định hợp lý — tiện cho test. */
export function voucher(overrides: Partial<Voucher> & Pick<Voucher, 'code'>): Voucher {
  return {
    percent: 10,
    maxDiscount: 20_000,
    minAmount: 0,
    kinds: [],
    cityCodes: [],
    validFrom: '2026-01-01',
    validTo: '2030-12-31',
    perCustomerPerDay: 1,
    ...overrides,
  };
}
