import 'dart:async';

import 'package:fake_async/fake_async.dart';
import 'package:test/test.dart';
import 'package:toi_noi_05/driver_location.dart';

final _t0 = DateTime.utc(2026, 11, 6, 8);

/// Điểm cách gốc [dLat] độ vĩ (0,0001 độ xấp xỉ 11 m).
Position _at(double dLat, int second) =>
    Position(21.0 + dLat, 105.8, _t0.add(Duration(seconds: second)));

void main() {
  group('distanceMeters', () {
    test('0,001 độ vĩ cách nhau khoảng 111 m', () {
      expect(distanceMeters(_at(0, 0), _at(0.001, 1)), closeTo(111.2, 0.5));
    });

    test('cùng một điểm cách nhau 0 m', () {
      expect(distanceMeters(_at(0, 0), _at(0, 5)), 0);
    });
  });

  group('movedMoreThan / onlyMoved', () {
    final p0 = _at(0, 0);
    final p1 = _at(0.0001, 1); // 11 m so với p0
    final p2 = _at(0.0002, 2); // 22 m so với p0
    final p3 = _at(0.001, 3); // 111 m so với p0
    final p4 = _at(0.0011, 4); // 11 m so với p3
    final p5 = _at(0.002, 5); // 111 m so với p3

    test('chỉ phát vị trí cách vị trí đã phát trước đó hơn 50 m', () async {
      final moved = onlyMoved(
        Stream.fromIterable([p0, p1, p2, p3, p4, p5]),
        50,
      );
      await expectLater(moved, emitsInOrder([p0, p3, p5, emitsDone]));
    });

    test('so với vị trí đã phát, không so với vị trí liền trước', () async {
      // Mỗi bước 11 m nhưng cộng dồn: p0, rồi chỉ phát khi tổng > 25 m.
      final steps = [for (var i = 0; i < 6; i++) _at(i * 0.0001, i)];
      final moved = onlyMoved(Stream.fromIterable(steps), 25);
      await expectLater(moved, emitsInOrder([steps[0], steps[3], emitsDone]));
    });

    test('stream rỗng thì kết thúc không phát gì', () async {
      await expectLater(onlyMoved(const Stream.empty(), 10), emitsDone);
    });

    test('nối với DriverLocationFeed qua timer ảo', () {
      fakeAsync((async) {
        final feed = DriverLocationFeed([p0, p1, p2, p3, p4, p5]);
        final seen = <Position>[];
        var done = false;
        feed
            .movedMoreThan(50, every: const Duration(seconds: 1))
            .listen(seen.add, onDone: () => done = true);

        async.elapse(const Duration(seconds: 6));
        expect(seen, [p0, p3, p5]);
        expect(done, isTrue);
      });
    });
  });

  group('DriverLocationFeed.positions', () {
    final route = [for (var i = 0; i < 6; i++) _at(i * 0.0001, i)];

    test('phát đúng thứ tự mỗi 1 giây rồi đóng stream', () {
      fakeAsync((async) {
        final feed = DriverLocationFeed(route);
        final seen = <Position>[];
        var done = false;
        feed
            .positions(every: const Duration(seconds: 1))
            .listen(seen.add, onDone: () => done = true);

        async.elapse(const Duration(milliseconds: 2500));
        expect(seen, route.sublist(0, 2));
        expect(done, isFalse);

        async.elapse(const Duration(seconds: 10));
        expect(seen, route);
        expect(done, isTrue);
        expect(async.pendingTimers, isEmpty);
      });
    });

    test('chưa có người nghe thì chưa có timer nào chạy', () {
      fakeAsync((async) {
        DriverLocationFeed(route).positions(every: const Duration(seconds: 1));
        expect(async.pendingTimers, isEmpty);
      });
    });

    test('cancel dừng timer và không nhận thêm vị trí', () {
      fakeAsync((async) {
        final feed = DriverLocationFeed(route);
        final seen = <Position>[];
        final subscription = feed
            .positions(every: const Duration(seconds: 1))
            .listen(seen.add);

        async.elapse(const Duration(seconds: 3));
        expect(seen, hasLength(3));
        expect(async.pendingTimers, isNotEmpty);

        subscription.cancel();
        expect(async.pendingTimers, isEmpty); // không rò rỉ timer

        async.elapse(const Duration(seconds: 10));
        expect(seen, hasLength(3));
      });
    });

    test('await for + break huỷ subscription, không còn timer', () {
      fakeAsync((async) {
        final feed = DriverLocationFeed(route);
        final seen = <Position>[];

        Future<void> consume() async {
          await for (final p in feed.positions(
            every: const Duration(seconds: 1),
          )) {
            seen.add(p);
            if (seen.length == 2) break;
          }
        }

        consume();
        async.elapse(const Duration(seconds: 5));
        expect(seen, route.sublist(0, 2));
        expect(async.pendingTimers, isEmpty);
      });
    });

    test('pause dừng nguồn phát, resume phát tiếp', () {
      fakeAsync((async) {
        final feed = DriverLocationFeed(route);
        final seen = <Position>[];
        final subscription = feed
            .positions(every: const Duration(seconds: 1))
            .listen(seen.add);

        async.elapse(const Duration(seconds: 2));
        subscription.pause();
        async.elapse(const Duration(seconds: 5));
        expect(seen, hasLength(2));

        subscription.resume();
        async.elapse(const Duration(seconds: 2));
        expect(seen, hasLength(4));
        subscription.cancel();
      });
    });

    test('single-subscription: nghe lần hai ném StateError', () {
      fakeAsync((async) {
        final stream = DriverLocationFeed(route)
            .positions(every: const Duration(seconds: 1));
        final first = stream.listen((_) {});
        expect(() => stream.listen((_) {}), throwsStateError);
        first.cancel();
      });
    });

    test('broadcast: nhiều người nghe cùng nhận được mọi vị trí', () {
      fakeAsync((async) {
        final stream = DriverLocationFeed(route)
            .positions(every: const Duration(seconds: 1), broadcast: true);
        final a = <Position>[];
        final b = <Position>[];
        stream.listen(a.add);
        stream.listen(b.add);

        async.elapse(const Duration(seconds: 6));
        expect(a, route);
        expect(b, route);
      });
    });

    test('broadcast: người nghe muộn bỏ lỡ vị trí đã phát', () {
      fakeAsync((async) {
        final stream = DriverLocationFeed(route)
            .positions(every: const Duration(seconds: 1), broadcast: true);
        final early = <Position>[];
        final lateOne = <Position>[];
        stream.listen(early.add);

        async.elapse(const Duration(seconds: 3));
        stream.listen(lateOne.add);
        async.elapse(const Duration(seconds: 3));

        expect(early, route);
        expect(lateOne, route.sublist(3));
      });
    });
  });
}
