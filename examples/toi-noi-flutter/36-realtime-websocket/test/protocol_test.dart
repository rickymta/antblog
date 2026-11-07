import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_36/realtime_protocol.dart';
import 'package:toi_noi_core/toi_noi_core.dart';
import 'package:toi_noi_fake_api/toi_noi_fake_api.dart' show tripEventToJson;

import 'support.dart';

void main() {
  group('parseServerFrame: từng loại frame trong hợp đồng', () {
    test('welcome', () {
      final frame = parseServerFrame(
        '{"type":"welcome","connectionId":"c-0001","userId":"U-1001"}',
      );
      expect(
        frame,
        isA<WelcomeFrame>().having((f) => f.userId, 'userId', 'U-1001'),
      );
    });

    test('subscribed: có logEpoch; bản cũ không có thì là null', () {
      final frame = parseServerFrame(
        '{"type":"subscribed","tripId":"T-1","latestSequence":3,"replayed":2,"logEpoch":"e1"}',
      );
      expect(
        frame,
        isA<SubscribedFrame>()
            .having((f) => f.latestSequence, 'latestSequence', 3)
            .having((f) => f.replayed, 'replayed', 2)
            .having((f) => f.logEpoch, 'logEpoch', 'e1'),
      );
      expect(
        parseServerFrame(
          '{"type":"subscribed","tripId":"T-1","latestSequence":0,"replayed":0}',
        ),
        isA<SubscribedFrame>().having((f) => f.logEpoch, 'logEpoch', isNull),
      );
    });

    test('subscribed kênh offers và offer', () {
      expect(
        parseServerFrame('{"type":"subscribed","channel":"offers"}'),
        isA<OffersSubscribedFrame>(),
      );
      final frame = parseServerFrame(
        '{"type":"offer","offerId":"O-0001","tripId":"T-1","serviceKind":"ride","distanceMeters":5000,"fare":25500,"currency":"VND","expiresAt":"2026-11-06T03:30:20.000Z"}',
      );
      final offer = (frame as OfferFrame).offer;
      expect(offer.serviceKind, ServiceKind.ride);
      expect(offer.fare, const Money(25500));
      expect(offer.distanceMeters, 5000);
      expect(offer.expiresAt, DateTime.utc(2026, 11, 6, 3, 30, 20));
    });

    test('event: sequence, trạng thái, thời điểm UTC và TripEvent', () {
      final frame = parseServerFrame(
        jsonEncode(
          eventFrame(
            'T-1',
            2,
            status: 'accepted',
            event: {'type': 'driverAccepted', 'driverId': 'D-7'},
          ),
        ),
      );

      final envelope = (frame as EventFrame).envelope;
      expect(envelope.tripId, 'T-1');
      expect(envelope.sequence, 2);
      expect(envelope.version, 2);
      expect(envelope.status, TripStatus.accepted);
      expect(envelope.occurredAt, DateTime.utc(2026, 11, 6, 3, 30));
      expect(envelope.event, const DriverAccepted('D-7'));
    });

    test('location: không có sequence, hai trường tuỳ chọn có thể vắng', () {
      final frame = parseServerFrame(
        jsonEncode({
          ...locationFrame('T-1', 10.7769, second: 5),
          'headingDegrees': 90,
        }),
      );

      final location = (frame as LocationFrame).location;
      expect(location.lat, 10.7769);
      expect(location.headingDegrees, 90.0);
      expect(location.speedMetersPerSecond, isNull);
      expect(location.at, DateTime.utc(2026, 11, 6, 3, 30, 5));
    });

    test('error, pong, unsubscribed', () {
      expect(
        parseServerFrame(
          '{"type":"error","code":"forbidden","message":"Not allowed","tripId":"T-1"}',
        ),
        isA<ErrorFrame>()
            .having((f) => f.code, 'code', 'forbidden')
            .having((f) => f.tripId, 'tripId', 'T-1'),
      );
      expect(parseServerFrame('{"type":"pong","id":"p-1"}'), isA<PongFrame>());
      expect(
        parseServerFrame('{"type":"unsubscribed","tripId":"T-1"}'),
        isA<UnsubscribedFrame>(),
      );
    });

    test('loại frame lạ và trường thừa: bỏ qua thay vì coi là lỗi', () {
      expect(
        parseServerFrame('{"type":"presence","users":3}'),
        isA<UnknownFrame>().having((f) => f.type, 'type', 'presence'),
      );
      final withExtra = parseServerFrame('{"type":"pong","id":1,"x":true}');
      expect(withExtra, isA<PongFrame>());
    });
  });

  group('parseServerFrame: dữ liệu xấu ném FormatException', () {
    final bad = <String, String>{
      'không phải JSON': 'xin chào',
      'JSON array': '[1,2]',
      'thiếu type': '{"tripId":"T-1"}',
      'event thiếu sequence': '{"type":"event","tripId":"T-1","occurredAt":"2026-11-06T03:30:00Z","status":"searching","event":{"type":"searchStarted"}}',
      'event thiếu version': jsonEncode(
        {...eventFrame('T-1', 1)}..remove('version'),
      ),
      'offer có serviceKind lạ': jsonEncode({
        'type': 'offer',
        'offerId': 'O-1',
        'tripId': 'T-1',
        'serviceKind': 'drone',
        'distanceMeters': 1,
        'fare': 1,
        'expiresAt': '2026-11-06T03:30:20.000Z',
      }),
      'sequence âm': jsonEncode(eventFrame('T-1', -1)),
      'sequence là chuỗi': jsonEncode({
        ...eventFrame('T-1', 1),
        'sequence': '1',
      }),
      'status lạ': jsonEncode(eventFrame('T-1', 1, status: 'flying')),
      'event.type lạ': jsonEncode(
        eventFrame('T-1', 1, event: {'type': 'teleport'}),
      ),
      'cancelled thiếu by': jsonEncode(
        eventFrame('T-1', 1, event: {'type': 'cancelled'}),
      ),
      'giờ không phải UTC': jsonEncode({
        ...eventFrame('T-1', 1),
        'occurredAt': '2026-11-06T10:30:00+07:00',
      }),
      'location thiếu lng': jsonEncode(
        {...locationFrame('T-1', 10.7)}..remove('lng'),
      ),
      'location lat là NaN dạng chuỗi': jsonEncode({
        ...locationFrame('T-1', 10.7),
        'lat': 'NaN',
      }),
    };

    bad.forEach((name, text) {
      test(
        name,
        () => expect(() => parseServerFrame(text), throwsFormatException),
      );
    });
  });

  group('hợp đồng với fake API', () {
    final events = <TripEvent>[
      const SearchStarted(),
      const DriverAccepted('D-7'),
      const DriverArrived(),
      const PickedUp(),
      const TripStarted(),
      const Completed(),
      const Cancelled(by: CancelledBy.customer),
      const Cancelled(by: CancelledBy.driver),
    ];

    for (final event in events) {
      test('đọc được JSON do server sinh cho ${event.runtimeType}', () {
        final json = tripEventToJson(event);
        expect(tripEventFromJson(json), event);
      });
    }

    test('bảy loại sự kiện của toi_noi_core đều có tên khác nhau', () {
      final names = {for (final e in events) tripEventToJson(e)['type']};
      expect(names, hasLength(7));
    });
  });

  group('frame client gửi', () {
    test('subscribe, unsubscribe, ping, offers đúng hợp đồng', () {
      expect(jsonDecode(encodeSubscribe('T-1', 4)), {
        'type': 'subscribe',
        'tripId': 'T-1',
        'lastSequence': 4,
      });
      expect(jsonDecode(encodeSubscribe('T-1', 4, logEpoch: 'e1')), {
        'type': 'subscribe',
        'tripId': 'T-1',
        'lastSequence': 4,
        'logEpoch': 'e1',
      });
      expect(jsonDecode(encodeSubscribeOffers()), {
        'type': 'subscribe',
        'channel': 'offers',
      });
      expect(jsonDecode(encodeUnsubscribeOffers()), {
        'type': 'unsubscribe',
        'channel': 'offers',
      });
      expect(jsonDecode(encodeUnsubscribe('T-1')), {
        'type': 'unsubscribe',
        'tripId': 'T-1',
      });
      expect(jsonDecode(encodePing('p-9')), {'type': 'ping', 'id': 'p-9'});
    });
  });
}
