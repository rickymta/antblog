import type { WeatherService } from '../app/ports';
import type { CityCode } from '../domain/types';

/** Dịch vụ thời tiết giả: test tự quyết thành phố nào đang mưa. */
export class FakeWeather implements WeatherService {
  private raining = new Set<CityCode>();

  constructor(rainingCities: CityCode[] = []) {
    this.setRaining(rainingCities);
  }

  setRaining(cities: CityCode[]): void {
    this.raining = new Set(cities);
  }

  async isRaining(cityCode: CityCode): Promise<boolean> {
    return this.raining.has(cityCode);
  }
}
