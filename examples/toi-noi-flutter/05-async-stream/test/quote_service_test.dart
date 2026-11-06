import 'dart:async';

import 'package:fake_async/fake_async.dart';
import 'package:test/test.dart';
import 'package:toi_noi_05/quote_service.dart';

/// Chạy [future] trong fakeAsync và gom kết quả hoặc lỗi vào [Outcome].
class Outcome<T> {
  T? value;
  Object? error;
  bool done = false;

  void watch(Future<T> future) {
    future.then(
      (v) {
        value = v;
        done = true;
      },
      onError: (Object e) {
        error = e;
        done = true;
      },
    );
  }
}

void main() {
  group('calculateFareVnd', () {
    test('ca biên của bảng giá', () {
      expect(calculateFareVnd(0), 12000);
      expect(calculateFareVnd(2), 12000);
      expect(calculateFareVnd(2.001), 12005); // 12.004,5 làm tròn lên
      expect(calculateFareVnd(5), 25500);
      expect(calculateFareVnd(100), 453000);
    });

    test('ngoài miền 0 đến 100 hoặc NaN ném ArgumentError', () {
      expect(() => calculateFareVnd(-0.001), throwsArgumentError);
      expect(() => calculateFareVnd(100.001), throwsArgumentError);
      expect(() => calculateFareVnd(double.nan), throwsArgumentError);
    });
  });

  group('QuoteService với fake_async', () {
    test('fetchFareVnd trả 25500 cho 5 km sau đúng 300 ms ảo', () {
      fakeAsync((async) {
        final outcome = Outcome<int>()..watch(QuoteService().fetchFareVnd(5));

        async.elapse(const Duration(milliseconds: 299));
        expect(outcome.done, isFalse);

        async.elapse(const Duration(milliseconds: 1));
        expect(outcome.value, 25500);
      });
    });

    test('chờ 10 giây ảo nhưng không tốn thời gian thật', () {
      final stopwatch = Stopwatch()..start();
      fakeAsync((async) {
        final outcome = Outcome<int>()
          ..watch(
            QuoteService(latency: const Duration(seconds: 10)).fetchFareVnd(5),
          );
        async.elapse(const Duration(seconds: 10));
        expect(outcome.value, 25500);
      });
      expect(stopwatch.elapsed, lessThan(const Duration(seconds: 1)));
    });

    test(
      'khoảng cách ngoài miền: lỗi đến qua Future dưới dạng QuoteException',
      () {
        fakeAsync((async) {
          final outcome = Outcome<int>()
            ..watch(QuoteService().fetchFareVnd(150));

          async.elapse(const Duration(milliseconds: 300));
          final error = outcome.error;
          expect(error, isA<QuoteException>());
          expect((error as QuoteException).cause, isA<ArgumentError>());
        });
      },
    );
  });

  group('timeout', () {
    test('timeout 100 ms ném TimeoutException trước khi service trả lời', () {
      fakeAsync((async) {
        final outcome = Outcome<int>()
          ..watch(
            fetchWithTimeout(
              QuoteService(),
              5,
              timeout: const Duration(milliseconds: 100),
            ),
          );

        async.elapse(const Duration(milliseconds: 100));
        expect(outcome.error, isA<TimeoutException>());
      });
    });

    test('timeout lớn hơn latency thì vẫn nhận được giá', () {
      fakeAsync((async) {
        final outcome = Outcome<int>()
          ..watch(
            fetchWithTimeout(
              QuoteService(),
              5,
              timeout: const Duration(seconds: 1),
            ),
          );

        async.elapse(const Duration(milliseconds: 300));
        expect(outcome.value, 25500);
      });
    });

    test('timeout không huỷ việc đang chạy: service vẫn xử lý xong', () {
      fakeAsync((async) {
        final service = QuoteService();
        final outcome = Outcome<int>()
          ..watch(
            fetchWithTimeout(
              service,
              5,
              timeout: const Duration(milliseconds: 100),
            ),
          );

        async.elapse(const Duration(milliseconds: 100));
        expect(outcome.error, isA<TimeoutException>());
        expect(service.completedCalls, 0);

        async.elapse(const Duration(milliseconds: 200));
        expect(service.completedCalls, 1); // việc vẫn chạy, kết quả bị bỏ
      });
    });
  });

  group('Future.wait và Future.any', () {
    test('Future.wait giữ thứ tự đầu vào dù thứ tự hoàn thành khác', () {
      fakeAsync((async) {
        final slow = QuoteService(latency: const Duration(milliseconds: 300));
        final fast = QuoteService(latency: const Duration(milliseconds: 100));
        final completionOrder = <String>[];

        final outcome = Outcome<List<int>>()
          ..watch(
            Future.wait([
              slow
                  .fetchFareVnd(5)
                  .whenComplete(() => completionOrder.add('slow')),
              fast
                  .fetchFareVnd(2)
                  .whenComplete(() => completionOrder.add('fast')),
            ]),
          );

        async.elapse(const Duration(milliseconds: 300));
        expect(completionOrder, ['fast', 'slow']);
        expect(outcome.value, [25500, 12000]); // đúng thứ tự gọi
      });
    });

    test('Future.wait mặc định chờ hết rồi mới báo lỗi xảy ra sớm nhất', () {
      fakeAsync((async) {
        final outcome = Outcome<List<int>>()
          ..watch(
            Future.wait([
              QuoteService(latency: const Duration(milliseconds: 100))
                  .fetchFareVnd(150), // lỗi sớm, ở 100 ms
              QuoteService(latency: const Duration(milliseconds: 300))
                  .fetchFareVnd(5),
            ]),
          );

        async.elapse(const Duration(milliseconds: 100));
        expect(outcome.done, isFalse); // còn chờ future chậm

        async.elapse(const Duration(milliseconds: 200));
        expect(outcome.error, isA<QuoteException>());
      });
    });

    test('Future.wait với eagerError: true báo lỗi ngay ở 100 ms', () {
      fakeAsync((async) {
        final outcome = Outcome<List<int>>()
          ..watch(
            Future.wait([
              QuoteService(latency: const Duration(milliseconds: 100))
                  .fetchFareVnd(150),
              QuoteService(latency: const Duration(milliseconds: 300))
                  .fetchFareVnd(5),
            ], eagerError: true),
          );

        async.elapse(const Duration(milliseconds: 100));
        expect(outcome.error, isA<QuoteException>());
        async.elapse(const Duration(milliseconds: 200)); // dọn timer còn lại
      });
    });

    test('gọi song song 3 quãng đường tốn 300 ms ảo, không phải 900 ms', () {
      fakeAsync((async) {
        final outcome = Outcome<List<int>>()
          ..watch(fetchFaresInParallel(QuoteService(), [2, 5, 10]));

        async.elapse(const Duration(milliseconds: 300));
        expect(outcome.value, [12000, 25500, 48000]);
      });
    });

    test('Future.wait thất bại khi một phần tử lỗi', () {
      fakeAsync((async) {
        final outcome = Outcome<List<int>>()
          ..watch(fetchFaresInParallel(QuoteService(), [2, 150, 10]));

        async.elapse(const Duration(milliseconds: 300));
        expect(outcome.error, isA<QuoteException>());
      });
    });

    test('Future.any lấy kết quả của nguồn nhanh nhất', () {
      fakeAsync((async) {
        final services = [
          QuoteService(latency: const Duration(milliseconds: 300)),
          QuoteService(latency: const Duration(milliseconds: 100)),
        ];
        final outcome = Outcome<int>()..watch(fetchFromFastest(services, 5));

        async.elapse(const Duration(milliseconds: 100));
        expect(outcome.value, 25500);
        expect(services[0].completedCalls, 0);

        async.elapse(const Duration(milliseconds: 200));
        expect(services[0].completedCalls, 1); // nguồn chậm vẫn chạy tiếp
      });
    });
  });
}
