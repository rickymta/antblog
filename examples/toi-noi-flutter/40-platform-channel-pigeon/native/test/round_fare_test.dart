import 'package:test/test.dart';
import 'package:toi_noi_round_fare/toi_noi_round_fare.dart';

void main() {
  test('hook đã build file C và @Native tìm thấy symbol', () {
    expect(toiNoiRoundFare(12005, 500), 12000);
  });

  test('nửa bước làm tròn lên, ví dụ cước 25.500 với bước 1.000', () {
    expect(toiNoiRoundFare(25500, 1000), 26000);
    expect(toiNoiRoundFare(25499, 1000), 25000);
  });

  test('kết quả vượt int64 trả -1', () {
    expect(toiNoiRoundFare(0x7fffffffffffffff, 1000), -1);
    expect(toiNoiRoundFare(0x7fffffffffffffff, 1), 0x7fffffffffffffff);
  });

  test('đầu vào sai trả -1', () {
    expect(toiNoiRoundFare(-1, 500), -1);
    expect(toiNoiRoundFare(1000, 0), -1);
    expect(toiNoiRoundFare(1000, -5), -1);
  });
}
