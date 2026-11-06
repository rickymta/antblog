import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_39/payment_flow.dart';
import 'package:toi_noi_39/payment_models.dart';
import 'package:toi_noi_39/payment_store.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'support/harness.dart';
import 'support/memory_gateway.dart';

void main() {
  late PaymentHarness h;
  late Trip trip;
  late PaymentFlow flow;

  setUp(() {
    h = PaymentHarness();
    trip = h.addTrip(); // 5 km: 25.500 VND, tính bằng calculateFare
    flow = h.flow();
  });

  Future<PaymentSession> startAndGetSession(PaymentFlow f) async {
    await f.start(tripId: trip.id, amount: trip.fare);
    return f.state.session!;
  }

  test('chuyến 5 km thu đúng 25.500 VND', () {
    expect(trip.fare, const Money(25500));
  });

  group('đường chính', () {
    test('tạo phiên, mở cổng, quay về, server xác nhận', () async {
      final seen = <Type>[];
      flow.changes.listen((s) => seen.add(s.runtimeType));

      final session = await startAndGetSession(flow);
      expect(flow.state, PaymentAwaitingCheckout(session));
      expect(h.launcher.opened, [session.checkoutUrl]);
      expect(session.amount, trip.fare);

      h.server.settle(session.id, PaymentStatus.succeeded);
      final handling = await flow.handleReturn(
        returnLink(session.id, status: 'success'),
      );

      expect(handling, ReturnHandling.handled);
      expect(flow.state, PaymentSucceeded(session));
      await pumpEventQueue();
      expect(seen, [
        PaymentCreating,
        PaymentAwaitingCheckout,
        PaymentConfirming,
        PaymentSucceeded,
      ]);
      expect(h.store.raw, isNull, reason: 'kết quả cuối thì xoá bản dang dở');
    });

    test('đã thành công thì start lại không tạo phiên mới', () async {
      final session = await startAndGetSession(flow);
      h.server.settle(session.id, PaymentStatus.succeeded);
      await flow.handleReturn(returnLink(session.id));

      await flow.start(tripId: trip.id, amount: trip.fare);

      expect(flow.state, isA<PaymentSucceeded>());
      expect(h.server.sessionCount, 1);
    });

    test('bấm hai lần liền chỉ tạo một phiên và mở cổng một lần', () async {
      await Future.wait([
        flow.start(tripId: trip.id, amount: trip.fare),
        flow.start(tripId: trip.id, amount: trip.fare),
      ]);
      expect(h.server.keysReceived, hasLength(1));
      expect(h.launcher.opened, hasLength(1));
    });
  });

  group('tham số deep link không phải bằng chứng', () {
    test(
      'status=success nhưng server nói chưa: hỏi có giới hạn rồi chờ xác nhận',
      () async {
        final session = await startAndGetSession(flow);

        final handling = await flow.handleReturn(
          returnLink(session.id, status: 'success'),
        );

        expect(handling, ReturnHandling.handled);
        expect(
          flow.state,
          PaymentPendingConfirmation(session, PendingReason.stillPending),
        );
        // Hỏi ngay một lần rồi chờ theo lịch: 1 + 4 lần, không hơn.
        expect(h.statusRequests, PollPolicy.standard.waits.length + 1);
        expect(h.waited, PollPolicy.standard.waits);
      },
    );

    test('sau đó khách trả thật: kiểm tra lại mới thành công', () async {
      final session = await startAndGetSession(flow);
      await flow.handleReturn(returnLink(session.id, status: 'success'));
      expect(flow.state, isA<PaymentPendingConfirmation>());

      h.server.settle(session.id, PaymentStatus.succeeded);
      await flow.refresh();

      expect(flow.state, PaymentSucceeded(session));
    });

    test('lịch hỏi tuỳ chỉnh được tôn trọng', () async {
      h = PaymentHarness(poll: const PollPolicy(waits: [Duration(seconds: 3)]));
      trip = h.addTrip();
      flow = h.flow();
      final session = await startAndGetSession(flow);

      await flow.handleReturn(returnLink(session.id));

      expect(h.statusRequests, 2);
      expect(h.waited, [const Duration(seconds: 3)]);
    });

    test('server thành công nhưng link nói cancel: vẫn thành công', () async {
      final session = await startAndGetSession(flow);
      h.server.settle(session.id, PaymentStatus.succeeded);
      await flow.handleReturn(returnLink(session.id, status: 'cancel'));
      expect(flow.state, PaymentSucceeded(session));
    });

    test('link của phiên khác bị bỏ qua, không gọi mạng', () async {
      final session = await startAndGetSession(flow);
      final before = h.statusRequests;

      final handling = await flow.handleReturn(
        returnLink('phien-gia-mao', status: 'success'),
      );

      expect(handling, ReturnHandling.unknownSession);
      expect(flow.state, PaymentAwaitingCheckout(session));
      expect(h.statusRequests, before);
    });

    test('link không phải của thanh toán thì không đụng tới state', () async {
      final session = await startAndGetSession(flow);
      for (final raw in [
        'toinoi://trip/T-1002',
        'https://evil.example/payment/return?session=${session.id}',
      ]) {
        expect(
          await flow.handleReturn(Uri.parse(raw)),
          ReturnHandling.notPaymentLink,
        );
      }
      expect(flow.state, PaymentAwaitingCheckout(session));
    });

    test(
      'link đến khi còn đang tạo phiên: bỏ qua, phiên vẫn tạo bình thường',
      () async {
        final gated = GatedGateway(MemoryGateway());
        final slow = PaymentFlow(
          gateway: gated,
          launcher: h.launcher,
          store: InMemoryPaymentStore(),
          newKey: () => 'key-1',
          now: () => h.now,
        );
        final starting = slow.start(tripId: trip.id, amount: trip.fare);
        await pumpEventQueue();
        expect(slow.state, isA<PaymentCreating>());

        expect(
          await slow.handleReturn(returnLink('mem-1', status: 'success')),
          ReturnHandling.unknownSession,
        );
        expect(slow.state, isA<PaymentCreating>());

        gated.open();
        await starting;
        expect(slow.state, isA<PaymentAwaitingCheckout>());
      },
    );

    test('link khi chưa có thanh toán nào: unknownSession', () async {
      expect(
        await flow.handleReturn(returnLink('s-1', status: 'success')),
        ReturnHandling.unknownSession,
      );
      expect(flow.state, const PaymentIdle());
    });
  });

  group('huỷ và hết hạn', () {
    for (final claimed in ['cancel', 'success', null]) {
      test(
        'khách huỷ ở cổng, link nói "${claimed ?? '(không có)'}": đã huỷ',
        () async {
          final session = await startAndGetSession(flow);
          h.server.settle(session.id, PaymentStatus.cancelled);

          await flow.handleReturn(returnLink(session.id, status: claimed));

          expect(flow.state, PaymentCancelled(session));
          expect(h.store.raw, isNull);
        },
      );
    }

    test('thất bại do ví từ chối', () async {
      final session = await startAndGetSession(flow);
      h.server.settle(session.id, PaymentStatus.failed);
      await flow.handleReturn(returnLink(session.id, status: 'success'));
      expect(flow.state, PaymentFailed(session));
    });

    test('quá hạn: link nói success vẫn là hết hạn', () async {
      final session = await startAndGetSession(flow);
      h.now = session.expiresAt.add(const Duration(minutes: 1));

      await flow.handleReturn(returnLink(session.id, status: 'success'));

      expect(flow.state, PaymentExpired(session));
    });

    test('mở lại cổng cho phiên quá hạn: hỏi server, không mở trang', () async {
      final session = await startAndGetSession(flow);
      h.now = session.expiresAt;

      await flow.reopenCheckout();

      expect(flow.state, PaymentExpired(session));
      expect(h.launcher.opened, hasLength(1));
    });

    test('thanh toán lại sau khi huỷ dùng khoá mới và phiên mới', () async {
      final first = await startAndGetSession(flow);
      h.server.settle(first.id, PaymentStatus.cancelled);
      await flow.handleReturn(returnLink(first.id));

      await flow.start(tripId: trip.id, amount: trip.fare);

      expect(h.server.keysReceived, ['key-1', 'key-2']);
      expect(flow.state.session!.id, isNot(first.id));
      expect(flow.state, isA<PaymentAwaitingCheckout>());
    });
  });

  group('deep link đến hai lần', () {
    test('đồng thời: một vòng hỏi, cả hai đều được xử lý', () async {
      final session = await startAndGetSession(flow);
      h.server.settle(session.id, PaymentStatus.succeeded);
      final link = returnLink(session.id, status: 'success');

      final results = await Future.wait([
        flow.handleReturn(link),
        flow.handleReturn(link),
      ]);

      expect(results, [ReturnHandling.handled, ReturnHandling.handled]);
      expect(h.statusRequests, 1);
      expect(flow.state, PaymentSucceeded(session));
    });

    test('lần hai sau khi đã chốt: không hỏi lại, kết quả không đổi', () async {
      final session = await startAndGetSession(flow);
      h.server.settle(session.id, PaymentStatus.succeeded);
      final link = returnLink(session.id, status: 'success');

      await flow.handleReturn(link);
      final after = h.statusRequests;
      final second = await flow.handleReturn(link);

      expect(second, ReturnHandling.handled);
      expect(h.statusRequests, after);
      expect(flow.state, PaymentSucceeded(session));
    });
  });

  group('đóng tab trình duyệt khi link quay về', () {
    test('link của phiên đang chờ: đóng tab đúng một lần', () async {
      final session = await startAndGetSession(flow);
      h.server.settle(session.id, PaymentStatus.succeeded);

      await flow.handleReturn(returnLink(session.id));
      await flow.handleReturn(returnLink(session.id));

      expect(h.launcher.closed, 1, reason: 'lần hai đã chốt, không đóng nữa');
    });

    test('link lạ hoặc không phải của thanh toán: không đóng', () async {
      await startAndGetSession(flow);

      await flow.handleReturn(returnLink('phien-gia-mao'));
      await flow.handleReturn(Uri.parse('toinoi://trip/T-1'));

      expect(h.launcher.closed, 0);
    });
  });

  group('mất mạng', () {
    test('mất mạng khi xác nhận là "đang chờ", không phải thất bại', () async {
      final session = await startAndGetSession(flow);
      h.server.settle(session.id, PaymentStatus.succeeded);
      h.network.offline = true;

      await flow.handleReturn(returnLink(session.id, status: 'success'));

      expect(
        flow.state,
        PaymentPendingConfirmation(session, PendingReason.unreachable),
      );
      expect(flow.state, isNot(isA<PaymentFinished>()));
      expect(h.store.raw, isNotNull, reason: 'còn dang dở thì còn lưu');

      h.network.offline = false;
      await flow.refresh();
      expect(flow.state, PaymentSucceeded(session));
    });

    test(
      'phản hồi tạo phiên bị rơi: thử lại cùng khoá, server chỉ có một phiên',
      () async {
        h.network.loseResponses = 1;

        await flow.start(tripId: trip.id, amount: trip.fare);

        expect(flow.state, isA<PaymentCreateFailed>());
        expect((flow.state as PaymentCreateFailed).retryable, isTrue);
        expect(h.server.sessionCount, 1, reason: 'server đã tạo trước khi rơi');

        await flow.start(tripId: trip.id, amount: trip.fare);

        expect(h.server.keysReceived, ['key-1', 'key-1']);
        expect(h.server.sessionCount, 1);
        expect(flow.state, isA<PaymentAwaitingCheckout>());
        expect(flow.state.session!.id, h.server.lastSessionId);
      },
    );

    test('mất mạng hẳn khi tạo phiên: thử lại được, khoá giữ nguyên', () async {
      h.network.offline = true;
      await flow.start(tripId: trip.id, amount: trip.fare);
      expect((flow.state as PaymentCreateFailed).retryable, isTrue);

      h.network.offline = false;
      await flow.start(tripId: trip.id, amount: trip.fare);

      expect(h.server.keysReceived, ['key-1']);
      expect(flow.state, isA<PaymentAwaitingCheckout>());
    });

    test(
      'server từ chối (sai số tiền): không thử lại được, lần sau khoá mới',
      () async {
        await flow.start(tripId: trip.id, amount: const Money(1000));

        final failed = flow.state as PaymentCreateFailed;
        expect(failed.retryable, isFalse);
        expect(h.store.raw, isNull);

        await flow.start(tripId: trip.id, amount: trip.fare);
        expect(h.server.keysReceived, ['key-1', 'key-2']);
        expect(flow.state, isA<PaymentAwaitingCheckout>());
      },
    );
  });

  group('app bị kill rồi mở lại (trạng thái bền qua PaymentStore)', () {
    test('không có gì dang dở: Idle, không gọi mạng', () async {
      await flow.restore();
      expect(flow.state, const PaymentIdle());
      expect(h.statusRequests, 0);
    });

    test('khách trả xong lúc app không chạy: mở lại thấy thành công', () async {
      final session = await startAndGetSession(flow);
      flow.dispose(); // kill
      h.server.settle(session.id, PaymentStatus.succeeded);

      final reopened = h.flow();
      await reopened.restore();

      expect(reopened.state, PaymentSucceeded(session));
      expect(h.store.raw, isNull);
    });

    test('app được mở lại bằng chính deep link quay về', () async {
      final session = await startAndGetSession(flow);
      flow.dispose();
      h.server.settle(session.id, PaymentStatus.succeeded);

      final reopened = h.flow();
      final handling = await reopened.handleReturn(
        returnLink(session.id, status: 'success'),
      );

      expect(handling, ReturnHandling.handled);
      expect(reopened.state, PaymentSucceeded(session));
    });

    test(
      'mở lại khi server vẫn pending: chờ xác nhận, mở lại cổng được',
      () async {
        final session = await startAndGetSession(flow);
        flow.dispose();

        final reopened = h.flow();
        await reopened.restore();
        expect(
          reopened.state,
          PaymentPendingConfirmation(session, PendingReason.stillPending),
        );

        await reopened.reopenCheckout();
        expect(reopened.state, PaymentAwaitingCheckout(session));
        expect(h.launcher.opened, [session.checkoutUrl, session.checkoutUrl]);
      },
    );

    test(
      'bị kill khi chưa biết server có tạo phiên chưa: thử lại cùng khoá',
      () async {
        h.network.loseResponses = 1;
        await flow.start(tripId: trip.id, amount: trip.fare);
        flow.dispose();

        final reopened = h.flow();
        await reopened.restore();
        final failed = reopened.state as PaymentCreateFailed;
        expect(failed.request.idempotencyKey, 'key-1');
        expect(failed.cause, isNull);

        await reopened.start(tripId: trip.id, amount: trip.fare);

        expect(h.server.keysReceived, ['key-1', 'key-1']);
        expect(h.server.sessionCount, 1);
        expect(reopened.state, isA<PaymentAwaitingCheckout>());
      },
    );

    test('kho chỉ chứa JSON của khoá, chuyến, số tiền và phiên', () async {
      final session = await startAndGetSession(flow);
      final raw = h.store.raw!;
      expect(raw, contains('key-1'));
      expect(raw, contains(session.id));
      expect(raw, contains('25500'));
    });
  });

  group('không mở được trang cổng', () {
    test('launchFailed rồi mở lại được', () async {
      h.launcher.result = false;
      final session = await startAndGetSession(flow);
      expect(flow.state, PaymentAwaitingCheckout(session, launchFailed: true));

      h.launcher.result = true;
      await flow.reopenCheckout();

      expect(flow.state, PaymentAwaitingCheckout(session));
    });
  });
}
