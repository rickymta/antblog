import 'package:mocktail/mocktail.dart';
import 'package:test/test.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'support/doubles.dart';

/// File này cố ý KHÔNG gọi `registerFallbacks()` ở đầu. Mỗi file test chạy
/// trong isolate riêng nên kho giá trị dự phòng của mocktail ở đây còn trống
/// cho tới khi test dưới đây tự đăng ký. Kho là trạng thái toàn cục của
/// isolate: vì vậy cả quá trình "thiếu, rồi đủ" nằm trong MỘT test, để thứ tự
/// chạy giữa các test không ảnh hưởng kết quả.
void main() {
  tearDown(resetMocktailState);

  test(
    'thiếu fallback thì StateError; đăng ký xong thì chạy và không gỡ được',
    () async {
      final repository = MockTripRepository();
      When<Future<Trip>> stub() => when(
        () => repository.requestTrip(any(), intentId: any(named: 'intentId')),
      );

      // Bước 1: chưa đăng ký. Lỗi nổ ngay khi `any()` được tính, tức là trước
      // khi mã sản phẩm chạy.
      expect(
        stub,
        throwsA(
          isA<StateError>().having(
            (e) => e.message,
            'message',
            allOf(
              contains('on a parameter of type `QuoteRequest`'),
              contains('registerFallbackValue was not previously called'),
            ),
          ),
        ),
      );
      resetMocktailState(); // `when` dở dang để lại cờ "đang stub"

      // Bước 2: đăng ký rồi dùng.
      registerFallbackValue(ride5km);
      stub().thenAnswer((_) async => tripFor(ride5km));
      expect(
        (await repository.requestTrip(food3km, intentId: 'i-1')).id,
        'T-1',
      );

      // Bước 3: reset trạng thái mocktail không gỡ fallback.
      resetMocktailState();
      expect(stub, returnsNormally);
    },
  );

  test('String đã có sẵn fallback: any() dùng được không cần đăng ký', () {
    final repository = MockTripRepository();
    when(() => repository.getTrip(any()))
        .thenAnswer((_) async => tripFor(ride5km));
    expect(repository.getTrip('T-1'), completion(tripFor(ride5km)));
  });
}
