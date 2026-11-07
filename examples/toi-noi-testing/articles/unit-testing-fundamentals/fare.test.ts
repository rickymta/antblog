import { describe, expect, it } from 'vitest';
import { calculateFare } from '../../src/domain/fare';

describe('calculateFare — năm quy tắc đầu tiên', () => {
  it('R-FARE-01/08: xe máy 2 km giờ thường trả 21.000 đ', () => {
    // Arrange: dữ liệu riêng cho test, không cần server hay đồng hồ.
    const input = { vehicle: 'bike' as const, distanceKm: 2, peak: false, raining: false };
    // Act
    const fare = calculateFare(input);
    // Assert: 12.000 + 4.500 × 2, tính độc lập từ SPEC.
    expect(fare.total).toBe(21_000);
  });

  it('R-FARE-01: ô tô 2 km trả 47.000 đ', () => {
    const fare = calculateFare({ vehicle: 'car', distanceKm: 2, peak: false, raining: false });
    expect(fare.total).toBe(47_000);
  });

  it('R-FARE-07: xe máy 0,5 km được nâng lên cước tối thiểu', () => {
    const fare = calculateFare({ vehicle: 'bike', distanceKm: 0.5, peak: false, raining: false });
    expect(fare).toMatchObject({ base: 14_250, minimumFare: 15_000, total: 15_000 });
  });

  it('R-FARE-04/08: cao điểm nhân 1,25 rồi làm tròn lên', () => {
    const fare = calculateFare({ vehicle: 'bike', distanceKm: 2, peak: true, raining: false });
    expect(fare).toMatchObject({ afterPeak: 26_250, total: 27_000 });
  });

  it('R-FARE-05: phụ phí mưa cộng sau hệ số cao điểm', () => {
    const fare = calculateFare({ vehicle: 'bike', distanceKm: 2, peak: true, raining: true });
    expect(fare).toMatchObject({ afterPeak: 26_250, rainSurcharge: 8_000, total: 35_000 });
  });
});
