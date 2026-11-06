import 'dart:isolate';

import 'package:flutter/foundation.dart' show compute;
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_41/trip_json.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

/// Giữ một `ReceivePort` làm field: không gửi được sang isolate khác.
class _LoaderWithPort {
  final ReceivePort port = ReceivePort();

  // Closure nhắc tới `port` nên bắt luôn `this`, mà `this` có ReceivePort.
  Future<int> countWrong(String json) =>
      Isolate.run(() => port.sendPort.hashCode + parseTrips(json).length);

  void close() => port.close();
}

void main() {
  late String json;

  setUpAll(() => json = generateTripsJson(2000));

  group('parseTrips và generateTripsJson', () {
    test('sinh đúng số bản ghi, cước đúng công thức', () {
      final trips = parseTrips(generateTripsJson(16));

      expect(trips, hasLength(16));
      // Khoảng cách xoay vòng 0,5 / 1,2 / 2 / 2,001 / 5 / ... / 100 km.
      expect(trips[0].fare, calculateFare(0.5));
      expect(trips[3].fare, const Money(12005));
      expect(trips[4].fare, const Money(25500));
      expect(trips[7].fare, const Money(453000));
    });

    test('JSON rỗng cho danh sách rỗng', () {
      expect(parseTrips(generateTripsJson(0)), isEmpty);
    });

    test('driverId khớp trạng thái (Trip tự kiểm)', () {
      final trips = parseTrips(json);

      expect(
        trips
            .where((t) => t.status == TripStatus.searching)
            .every((t) => t.driverId == null),
        isTrue,
      );
      expect(
        trips
            .where((t) => t.status == TripStatus.completed)
            .every((t) => t.driverId != null),
        isTrue,
      );
    });

    test('gốc không phải mảng ném FormatException', () {
      expect(() => parseTrips('{"a":1}'), throwsFormatException);
    });

    test('bản ghi sai nêu số thứ tự', () {
      const bad =
          '[{"id":"T1","service":"ride","status":"completed",'
          '"distanceMeters":5000,"fareVnd":25500,"driverId":"D1",'
          '"requestedAt":"2026-01-01T00:00:00.000Z"},'
          '{"id":"T2","service":"ride","status":"bay-len-troi"}]';

      expect(
        () => parseTrips(bad),
        throwsA(
          isA<FormatException>().having(
            (e) => e.message,
            'message',
            contains('#1'),
          ),
        ),
      );
    });

    test('tài xế không khớp trạng thái bị chặn bởi Trip', () {
      const bad =
          '[{"id":"T1","service":"ride","status":"searching",'
          '"distanceMeters":5000,"fareVnd":25500,"driverId":"D1",'
          '"requestedAt":"2026-01-01T00:00:00.000Z"}]';

      expect(() => parseTrips(bad), throwsFormatException);
    });
  });

  group('Isolate.run', () {
    test('cho đúng kết quả như chạy trên isolate hiện tại', () async {
      final inIsolate = await parseTripsInIsolate(json);

      expect(inIsolate, parseTrips(json));
    });

    test('Trip và Money đi qua ranh giới isolate nguyên vẹn', () async {
      final trips = await parseTripsInIsolate(generateTripsJson(8));

      expect(trips.first, isA<Trip>());
      expect(trips[7].fare, const Money(453000));
      expect(trips[0].requestedAt.isUtc, isTrue);
    });

    test('lỗi trong isolate ném lại đúng kiểu ở nơi gọi', () {
      expect(
        parseTripsInIsolate('[{"id":1}]'),
        throwsA(
          isA<FormatException>().having(
            (e) => e.message,
            'message',
            contains('#0'),
          ),
        ),
      );
    });

    test(
      'dữ liệu mutable bị SAO CHÉP: sửa ở isolate kia không ảnh hưởng',
      () async {
        final fares = <int>[12000, 25500];

        final lengthInside = await Isolate.run(() {
          fares.add(453000);
          return fares.length;
        });

        expect(lengthInside, 3);
        expect(fares, [12000, 25500]);
      },
    );

    test('closure bắt this có ReceivePort thì không gửi được', () async {
      final loader = _LoaderWithPort();
      addTearDown(loader.close);

      await expectLater(
        loader.countWrong(generateTripsJson(4)),
        throwsA(
          isA<ArgumentError>().having(
            (e) => e.toString(),
            'message',
            contains('unsendable'),
          ),
        ),
      );
    });

    test('compute tương đương Isolate.run(() => fun(message))', () async {
      final viaCompute = await compute(parseTrips, json);

      expect(viaCompute, parseTrips(json));
    });
  });

  group('main isolate có bị chặn không', () {
    // Không đo mili giây (phụ thuộc máy); chỉ kiểm thứ tự xử lý của event
    // loop: một Timer đã tới hạn chạy trước hay sau khi parse xong.
    test('parse đồng bộ: Timer phải chờ parse xong', () async {
      final order = <String>[];
      final timer = Future<void>(() => order.add('timer'));

      parseTrips(generateTripsJson(5000));
      order.add('parse xong');
      await timer;

      expect(order, ['parse xong', 'timer']);
    });

    test('parse ở isolate khác: Timer chạy trong lúc chờ', () async {
      final order = <String>[];
      final timer = Future<void>(() => order.add('timer'));

      final parsing = parseTripsInIsolate(generateTripsJson(5000));
      await parsing;
      order.add('parse xong');
      await timer;

      expect(order, ['timer', 'parse xong']);
    });
  });
}
