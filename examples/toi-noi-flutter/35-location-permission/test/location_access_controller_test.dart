import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_35/location_access.dart';
import 'package:toi_noi_35/location_access_controller.dart';
import 'package:toi_noi_35/location_models.dart';

import 'support/fake_location_source.dart';

LocationAccessController make(
  FakeLocationSource source, {
  AppRole role = AppRole.customer,
  AskedStore? store,
}) => LocationAccessController(
  source: source,
  store: store ?? MemoryAskedStore(),
  role: role,
);

void main() {
  group('xin quyền foreground', () {
    test('refresh khi mở màn hình không hiện hộp thoại nào', () async {
      final source = FakeLocationSource();
      final c = make(source);
      await c.refresh();
      expect(c.value, isA<NotAsked>());
      expect(source.foregroundRequests, 0);
    });

    test('NotAsked -> Asking -> WhileInUse khi người dùng đồng ý', () async {
      final source = FakeLocationSource()
        ..foregroundAnswers.add(SystemPermission.whileInUse);
      final c = make(source);
      final seen = <Type>[];
      c.addListener(() => seen.add(c.value.runtimeType));
      await c.refresh();
      await c.requestForeground();
      expect(seen, containsAllInOrder([Asking, WhileInUse]));
      expect(c.value, const WhileInUse(LocationPrecision.precise));
      expect(source.foregroundRequests, 1);
    });

    test(
      'Android: từ chối lần một là Denied, lần hai là DeniedForever',
      () async {
        // checkPermission của Android không bao giờ trả deniedForever: chỉ kết
        // quả của lần xin thứ hai mới báo. Controller phải giữ kết quả đó.
        final source = FakeLocationSource(androidLike: true)
          ..foregroundAnswers.addAll([
            SystemPermission.denied,
            SystemPermission.deniedForever,
          ]);
        final c = make(source);
        await c.refresh();

        await c.requestForeground();
        expect(c.value, isA<Denied>(), reason: 'app nhớ đã hỏi một lần');

        await c.requestForeground();
        expect(c.value, isA<DeniedForever>());
        expect(source.foregroundRequests, 2);

        await c.refresh();
        expect(
          c.value,
          isA<DeniedForever>(),
          reason: 'checkPermission vẫn báo denied, cờ đã lưu giữ trạng thái',
        );
        await c.requestForeground();
        expect(source.foregroundRequests, 2, reason: 'không còn hộp thoại nào');
      },
    );

    test('iOS: từ chối lần đầu đã là DeniedForever', () async {
      final source = FakeLocationSource()
        ..foregroundAnswers.add(SystemPermission.deniedForever);
      final c = make(source);
      await c.refresh();
      await c.requestForeground();
      expect(c.value, isA<DeniedForever>());
    });

    test('Android: được cấp trong Cài đặt thì xoá cờ bị chặn', () async {
      final source = FakeLocationSource(androidLike: true)
        ..foregroundAnswers.addAll([
          SystemPermission.denied,
          SystemPermission.deniedForever,
        ]);
      final c = make(source);
      await c.refresh();
      await c.requestForeground();
      await c.requestForeground();
      expect(c.value, isA<DeniedForever>());

      source.permission = SystemPermission.whileInUse;
      await c.refresh();
      expect(c.value, isA<WhileInUse>());

      source.permission = SystemPermission.denied;
      await c.refresh();
      expect(c.value, isA<Denied>());
    });

    test('DeniedForever: gọi tiếp không còn mở hộp thoại', () async {
      final source = FakeLocationSource(
        permission: SystemPermission.deniedForever,
      );
      final c = make(source);
      await c.refresh();
      await c.requestForeground();
      await c.requestForeground();
      expect(source.foregroundRequests, 0);
      expect(c.value, isA<DeniedForever>());
    });

    test('dịch vụ vị trí tắt: không xin quyền, chỉ báo ServiceOff', () async {
      final source = FakeLocationSource(serviceEnabled: false);
      final c = make(source);
      await c.refresh();
      await c.requestForeground();
      expect(c.value, isA<ServiceOff>());
      expect(source.foregroundRequests, 0);
    });

    test('quay lại từ Cài đặt: refresh bắt được quyền mới', () async {
      final source = FakeLocationSource(
        permission: SystemPermission.deniedForever,
      );
      final c = make(source);
      await c.refresh();
      expect(c.value, isA<DeniedForever>());

      source.permission = SystemPermission.whileInUse;
      await c.refresh();
      expect(c.value, isA<WhileInUse>());
    });

    test('refresh bị bỏ qua khi hộp thoại đang hiện', () async {
      final gate = Completer<void>();
      final source = FakeLocationSource()
        ..foregroundGate = gate
        ..foregroundAnswers.add(SystemPermission.whileInUse);
      final c = make(source);
      await c.refresh();

      final pending = c.requestForeground();
      await Future<void>.delayed(Duration.zero);
      expect(c.value, isA<Asking>());

      await c.refresh();
      expect(
        c.value,
        isA<Asking>(),
        reason: 'không ghi đè trạng thái đang hỏi',
      );

      gate.complete();
      await pending;
      expect(c.value, isA<WhileInUse>());
    });

    test('hai lần bấm liên tiếp chỉ mở một hộp thoại', () async {
      final gate = Completer<void>();
      final source = FakeLocationSource()
        ..foregroundGate = gate
        ..foregroundAnswers.add(SystemPermission.whileInUse);
      final c = make(source);
      await c.refresh();

      final first = c.requestForeground();
      await Future<void>.delayed(Duration.zero);
      await c.requestForeground();
      gate.complete();
      await first;
      expect(source.foregroundRequests, 1);
    });

    test(
      'thiếu khai báo native: ném lỗi cấu hình, không đánh dấu đã hỏi',
      () async {
        final source = FakeLocationSource()
          ..foregroundError = const LocationConfigurationException('thiếu key');
        final store = MemoryAskedStore();
        final c = make(source, store: store);
        await c.refresh();
        await expectLater(
          c.requestForeground(),
          throwsA(isA<LocationConfigurationException>()),
        );
        expect(await store.asked(AskTarget.foreground), isFalse);
        expect(c.value, isA<NotAsked>());
      },
    );

    test(
      'hộp thoại khác đang hiện: không đánh dấu đã hỏi, không ném lỗi',
      () async {
        final source = FakeLocationSource()
          ..foregroundError = const LocationRequestInProgressException('bận');
        final store = MemoryAskedStore();
        final c = make(source, store: store);
        await c.refresh();
        await c.requestForeground();
        expect(await store.asked(AskTarget.foreground), isFalse);
        expect(c.value, isA<NotAsked>());
      },
    );
  });

  group('xin nền (app tài xế)', () {
    test('WhileInUse -> Asking -> Always khi đồng ý', () async {
      final source = FakeLocationSource(permission: SystemPermission.whileInUse)
        ..backgroundAnswers.add(SystemPermission.always);
      final c = make(source, role: AppRole.driver);
      await c.refresh();
      await c.requestBackground();
      expect(c.value, const Always(LocationPrecision.precise));
      expect(source.backgroundRequests, 1);
    });

    test(
      'từ chối nền: vẫn WhileInUse và nhớ đã xin, không làm phiền lại',
      () async {
        final source = FakeLocationSource(
          permission: SystemPermission.whileInUse,
        );
        final c = make(source, role: AppRole.driver);
        await c.refresh();
        await c.requestBackground();
        expect(
          c.value,
          const WhileInUse(LocationPrecision.precise, backgroundAsked: true),
        );
        expect(
          adviceFor(c.value, AppRole.driver, tripActive: true),
          LocationAdvice.ready,
        );
      },
    );

    test('app khách không bao giờ xin nền', () async {
      final source = FakeLocationSource(
        permission: SystemPermission.whileInUse,
      );
      final c = make(source);
      await c.refresh();
      await c.requestBackground();
      expect(source.backgroundRequests, 0);
      expect(c.value, isA<WhileInUse>());
    });

    test('nền tảng không có bước xin nền (Android): không xin', () async {
      final source = FakeLocationSource(
        permission: SystemPermission.whileInUse,
        backgroundSupported: false,
      );
      final c = make(source, role: AppRole.driver);
      await c.refresh();
      await c.requestBackground();
      expect(source.backgroundRequests, 0);
      expect(c.backgroundSupported, isFalse);
    });

    test('chưa có quyền foreground thì không xin nền', () async {
      final source = FakeLocationSource();
      final c = make(source, role: AppRole.driver);
      await c.refresh();
      await c.requestBackground();
      expect(source.backgroundRequests, 0);
    });
  });

  group('độ chính xác', () {
    test('reduced -> xin chính xác -> precise', () async {
      final source = FakeLocationSource(
        permission: SystemPermission.whileInUse,
        precisionValue: LocationPrecision.reduced,
      )..preciseAnswer = LocationPrecision.precise;
      final c = make(source);
      await c.refresh();
      expect(c.value.precision, LocationPrecision.reduced);

      await c.requestPrecise();
      expect(c.value.precision, LocationPrecision.precise);
      expect(source.preciseRequests, 1);
    });

    test('đã chính xác hoặc chưa có quyền: không xin', () async {
      final source = FakeLocationSource(
        permission: SystemPermission.whileInUse,
      );
      final c = make(source);
      await c.refresh();
      await c.requestPrecise();

      final none = FakeLocationSource();
      final c2 = make(none);
      await c2.refresh();
      await c2.requestPrecise();

      expect(source.preciseRequests + none.preciseRequests, 0);
    });
  });
}
