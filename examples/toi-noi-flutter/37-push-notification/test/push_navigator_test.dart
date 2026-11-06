import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_37/push_message.dart';
import 'package:toi_noi_37/push_navigator.dart';
import 'package:toi_noi_37/push_router.dart';

import 'support.dart';

void main() {
  late FakePushSource source;
  late FakeLocalNotifier notifier;
  late List<String> opened;
  late List<String> rejected;
  late PushNavigator navigator;

  setUp(() {
    source = FakePushSource();
    notifier = FakeLocalNotifier();
    opened = [];
    rejected = [];
    navigator = PushNavigator(
      source: source,
      notifier: notifier,
      open: opened.add,
      onRejected: rejected.add,
    );
  });

  tearDown(() => navigator.dispose());

  group('foreground', () {
    test('hiện thông báo cục bộ và KHÔNG tự chuyển màn hình', () async {
      await navigator.start();
      source.emitForeground(tripMessage('T-1002'));
      await pumpEventQueue();

      expect(opened, isEmpty);
      expect(notifier.shown, hasLength(1));
      expect(notifier.shown.single.title, 'Tài xế đã nhận chuyến');
      expect(
        const PushRouter().decideFromPayload(notifier.shown.single.payload),
        const OpenTrip('T-1002'),
      );
    });

    test('payload lạ: không hiện gì, báo lý do không kèm giá trị', () async {
      await navigator.start();
      source.emitForeground(
        const PushMessage(data: {'type': 'promo', 'tripId': 'T-1'}),
      );
      await pumpEventQueue();

      expect(notifier.shown, isEmpty);
      expect(rejected, ['type không được hỗ trợ']);
    });

    test('payload của thông báo cục bộ không mang khoá thừa', () async {
      await navigator.start();
      source.emitForeground(
        tripMessage('T-1002', extra: {'driverPhone': 'khong-duoc-xuat-hien'}),
      );
      await pumpEventQueue();
      expect(notifier.shown.single.payload, isNot(contains('driverPhone')));
    });

    test(
      'mỗi thông báo có id riêng, thông báo sau không đè thông báo trước',
      () async {
        await navigator.start();
        source.emitForeground(tripMessage('T-1'));
        source.emitForeground(tripMessage('T-2'));
        await pumpEventQueue();
        expect(notifier.shown.map((n) => n.id).toSet(), hasLength(2));
      },
    );
  });

  group('background (onMessageOpenedApp)', () {
    test('chạm thông báo thì mở đúng chuyến', () async {
      await navigator.start();
      source.tapFromBackground(tripMessage('T-1002'));
      await pumpEventQueue();
      expect(opened, ['/trip/T-1002']);
    });

    test('payload lạ thì không mở gì', () async {
      await navigator.start();
      source.tapFromBackground(
        const PushMessage(data: {'type': 'trip_update', 'tripId': '../x'}),
      );
      await pumpEventQueue();
      expect(opened, isEmpty);
      expect(rejected, ['tripId thiếu hoặc sai định dạng']);
    });
  });

  group('terminated (getInitialMessage)', () {
    test('tin khởi chạy app được mở ngay khi start', () async {
      source.initialMessage = tripMessage('T-1002');
      await navigator.start();
      expect(opened, ['/trip/T-1002']);
    });

    test('plugin chỉ trả tin một lần: start lần hai không mở lại', () async {
      source.initialMessage = tripMessage('T-1002');
      await navigator.start();
      await navigator.dispose();
      await navigator.start();
      expect(opened, ['/trip/T-1002']);
      expect(source.takeInitialCalls, 2);
    });

    test('start gọi hai lần liền không đăng ký nghe hai lần', () async {
      await navigator.start();
      await navigator.start();
      source.tapFromBackground(tripMessage('T-1'));
      await pumpEventQueue();
      expect(opened, ['/trip/T-1']);
      expect(source.takeInitialCalls, 1);
    });

    test('tin khởi chạy lạ thì bị bỏ qua', () async {
      source.initialMessage = const PushMessage(data: {'x': '1'});
      await navigator.start();
      expect(opened, isEmpty);
      expect(rejected, ['thiếu type']);
    });
  });

  group('thông báo cục bộ do chính app hiện', () {
    test('chạm khi app đang chạy thì mở chuyến', () async {
      await navigator.start();
      notifier.tap(const OpenTrip('T-7').toPayload());
      await pumpEventQueue();
      expect(opened, ['/trip/T-7']);
    });

    test(
      'chạm vào thông báo cục bộ khi app bị tắt: mở qua launchPayload',
      () async {
        notifier.launch = const OpenTrip('T-8').toPayload();
        await navigator.start();
        expect(opened, ['/trip/T-8']);
      },
    );

    test('đã có tin FCM khởi chạy thì không mở thêm launchPayload', () async {
      source.initialMessage = tripMessage('T-1');
      notifier.launch = const OpenTrip('T-2').toPayload();
      await navigator.start();
      expect(opened, ['/trip/T-1']);
    });

    test('payload cục bộ hỏng thì bị bỏ qua', () async {
      await navigator.start();
      notifier.tap('{"type":"trip_update"');
      await pumpEventQueue();
      expect(opened, isEmpty);
      expect(rejected, ['payload không phải JSON']);
    });
  });

  test('sau dispose không còn phản ứng với tin nào', () async {
    await navigator.start();
    await navigator.dispose();
    source.emitForeground(tripMessage('T-1'));
    source.tapFromBackground(tripMessage('T-1'));
    notifier.tap(const OpenTrip('T-1').toPayload());
    await pumpEventQueue();
    expect(opened, isEmpty);
    expect(notifier.shown, isEmpty);
  });
}
