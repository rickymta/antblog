import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_39/payment_gateway.dart';
import 'package:toi_noi_39/payment_models.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'support/harness.dart';

Matcher _rejected(int status) => isA<PaymentGatewayException>()
    .having((e) => e.kind, 'kind', GatewayErrorKind.rejected)
    .having((e) => e.statusCode, 'statusCode', status);

void main() {
  late PaymentHarness h;
  late PaymentGateway gateway;
  late Trip trip;

  setUp(() {
    h = PaymentHarness();
    gateway = HttpPaymentGateway(h.dio);
    trip = h.addTrip();
  });

  Future<PaymentSession> create(String key, {Money? amount}) =>
      gateway.createSession(
        tripId: trip.id,
        amount: amount ?? trip.fare,
        idempotencyKey: key,
      );

  group('idempotency của server giả', () {
    test('cùng khoá, cùng nội dung: đúng phiên cũ, chỉ có một phiên', () async {
      final first = await create('k-1');
      final replay = await create('k-1');
      expect(replay, first);
      expect(h.server.sessionCount, 1);
    });

    test('khác khoá thì là hai phiên: đó là thứ khoá ngăn lại', () async {
      final a = await create('k-1');
      final b = await create('k-2');
      expect(a.id, isNot(b.id));
      expect(h.server.sessionCount, 2);
    });

    test('cùng khoá nhưng số tiền khác: 409', () async {
      await create('k-1');
      await expectLater(
        create('k-1', amount: const Money(1000)),
        throwsA(_rejected(409)),
      );
    });

    test('khoá không hợp lệ: 422', () async {
      await expectLater(create('khoá có dấu cách'), throwsA(_rejected(422)));
    });

    test('dùng lại khoá của phiên đã huỷ trả về chính phiên đã huỷ', () async {
      final session = await create('k-1');
      h.server.settle(session.id, PaymentStatus.cancelled);
      final replay = await create('k-1');
      expect(replay.id, session.id);
      expect(
        await gateway.fetchStatus(replay.id),
        PaymentStatus.cancelled,
        reason: 'thử lại sau khi huỷ phải dùng khoá mới',
      );
    });
  });

  group('số tiền và chuyến do server kiểm', () {
    test('số tiền khác cước của chuyến: 422', () async {
      await expectLater(
        create('k-1', amount: const Money(100)),
        throwsA(_rejected(422)),
      );
      expect(h.server.sessionCount, 0);
    });

    test('chuyến không tồn tại: 404', () async {
      await expectLater(
        gateway.createSession(
          tripId: 'khong-co',
          amount: trip.fare,
          idempotencyKey: 'k-1',
        ),
        throwsA(_rejected(404)),
      );
    });

    test('chuyến đã trả xong: khoá mới cũng bị 409', () async {
      final session = await create('k-1');
      h.server.settle(session.id, PaymentStatus.succeeded);
      await expectLater(create('k-2'), throwsA(_rejected(409)));
    });
  });

  group('trạng thái phiên', () {
    test('pending cho tới khi cổng chốt', () async {
      final session = await create('k-1');
      expect(await gateway.fetchStatus(session.id), PaymentStatus.pending);
      h.server.settle(session.id, PaymentStatus.succeeded);
      expect(await gateway.fetchStatus(session.id), PaymentStatus.succeeded);
    });

    test('hết hạn theo đồng hồ của server', () async {
      final session = await create('k-1');
      h.now = session.expiresAt.subtract(const Duration(seconds: 1));
      expect(await gateway.fetchStatus(session.id), PaymentStatus.pending);
      h.now = session.expiresAt;
      expect(await gateway.fetchStatus(session.id), PaymentStatus.expired);
    });

    test(
      'đã chốt thì không đổi: trả muộn sau khi hết hạn không có tác dụng',
      () async {
        final session = await create('k-1');
        h.now = session.expiresAt;
        expect(h.server.settle(session.id, PaymentStatus.succeeded), isFalse);
        expect(await gateway.fetchStatus(session.id), PaymentStatus.expired);
      },
    );

    test('phiên lạ: 404', () async {
      await expectLater(
        gateway.fetchStatus('khong-co'),
        throwsA(_rejected(404)),
      );
    });
  });

  group('HttpPaymentGateway phân loại lỗi', () {
    test('mất mạng là unreachable và thử lại được', () async {
      h.network.offline = true;
      await expectLater(
        create('k-1'),
        throwsA(
          isA<PaymentGatewayException>()
              .having((e) => e.kind, 'kind', GatewayErrorKind.unreachable)
              .having((e) => e.retryable, 'retryable', isTrue),
        ),
      );
    });

    test('4xx là rejected và không thử lại được', () async {
      await expectLater(
        create('k-1', amount: const Money(1)),
        throwsA(
          isA<PaymentGatewayException>().having(
            (e) => e.retryable,
            'retryable',
            isFalse,
          ),
        ),
      );
    });
  });
}
