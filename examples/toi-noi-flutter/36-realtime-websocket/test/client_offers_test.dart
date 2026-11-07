import 'package:fake_async/fake_async.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'support.dart';

void main() {
  const subscribeOffers = {'type': 'subscribe', 'channel': 'offers'};

  test('subscribeOffers trước khi connected: gửi sau welcome, nhận lời mời có cước đúng', () {
    fakeAsync((async) {
      final transport = FakeTransport();
      final client = buildClient(
        transport,
        heartbeatInterval: const Duration(hours: 1),
      );
      final offers = collect(client.offers);

      client.subscribeOffers();
      client.connect();
      settle(async);
      expect(transport.last.sentOfType('subscribe'), [subscribeOffers]);

      transport.last.serverSend(offerFrame('O-0001'));
      settle(async);

      expect(offers, hasLength(1));
      final offer = offers.single;
      expect(offer.offerId, 'O-0001');
      expect(offer.serviceKind, ServiceKind.ride);
      expect(offer.fare, const Money(25500));
      expect(offer.expiresAt, DateTime.utc(2026, 11, 6, 3, 30, 20));
    });
  });

  test('nối lại đăng ký lại kênh offers', () {
    fakeAsync((async) {
      final transport = FakeTransport();
      final client = buildClient(
        transport,
        heartbeatInterval: const Duration(hours: 1),
      );
      client.subscribeOffers();
      client.connect();
      settle(async);

      transport.last.serverDrop();
      settle(async);
      async.elapse(client.state.nextRetryIn!);

      expect(transport.connections, hasLength(2));
      expect(transport.last.sentOfType('subscribe'), [subscribeOffers]);
    });
  });

  test('unsubscribeOffers: gửi frame, không đăng ký lại khi nối lại, lời mời tới muộn bị bỏ', () {
    fakeAsync((async) {
      final transport = FakeTransport();
      final client = buildClient(
        transport,
        heartbeatInterval: const Duration(hours: 1),
      );
      final offers = collect(client.offers);
      client.subscribeOffers();
      client.connect();
      settle(async);

      client.unsubscribeOffers();
      transport.last.serverSend(offerFrame('O-0001')); // đã trên đường tới
      settle(async);
      transport.last.serverDrop();
      settle(async);
      async.elapse(client.state.nextRetryIn!);

      expect(transport.connections.first.sentOfType('unsubscribe'), [
        {'type': 'unsubscribe', 'channel': 'offers'},
      ]);
      expect(transport.last.sentOfType('subscribe'), isEmpty);
      expect(offers, isEmpty);
    });
  });

  test('khách (không phải tài xế) bị forbidden ở kênh offers: báo issue, không xin lại khi nối lại', () {
    fakeAsync((async) {
      final transport = FakeTransport();
      final client = buildClient(
        transport,
        heartbeatInterval: const Duration(hours: 1),
      );
      final issues = collect(client.issues);
      client.subscribeOffers();
      client.connect();
      settle(async);

      transport.last.serverSend({
        'type': 'error',
        'code': 'forbidden',
        'message': 'Only drivers have offers',
        'channel': 'offers',
      });
      settle(async);
      transport.last.serverDrop();
      settle(async);
      async.elapse(client.state.nextRetryIn!);

      expect(issues.single.code, 'forbidden');
      expect(transport.last.sentOfType('subscribe'), isEmpty);
    });
  });
}
