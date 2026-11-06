import 'package:test/test.dart';
import 'package:toi_noi_05/event_loop_demo.dart';
import 'package:toi_noi_05/heavy.dart';

void main() {
  group('event loop', () {
    test('thứ tự: sync, microtask, event', () async {
      expect(await traceEventLoop(), ['sync', 'microtask', 'event']);
    });

    test(
      'thứ tự đầy đủ: sync, microtask theo đăng ký, rồi event theo đăng ký',
      () async {
        expect(await traceQueues(), [
          'sync',
          'scheduleMicrotask',
          'Future.microtask',
          'Timer.run',
          'Future',
        ]);
      },
    );

    test('async chạy đồng bộ tới await đầu tiên', () async {
      expect(await traceAwaitOrder(), [
        'work: trước await',
        'caller: sau khi gọi work',
        'work: sau await',
      ]);
    });
  });

  group('Isolate.run', () {
    test('sumInIsolate cho kết quả giống công thức n(n+1)/2', () async {
      const n = 1000000;
      expect(await sumInIsolate(n), n * (n + 1) ~/ 2);
    });

    test('kết quả giống tính trên main isolate', () async {
      expect(await sumInIsolate(12345), sumUpTo(12345));
    });

    test('lỗi trong isolate được ném lại cho người gọi', () async {
      await expectLater(sumInIsolate(-1), throwsArgumentError);
    });
  });
}
