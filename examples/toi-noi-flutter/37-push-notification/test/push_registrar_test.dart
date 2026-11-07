import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_37/device_api.dart';
import 'package:toi_noi_37/push_message.dart';
import 'package:toi_noi_37/push_registrar.dart';

import 'support.dart';

void main() {
  late FakePushSource source;
  late FakeDeviceApi api;
  late PushRegistrar registrar;
  late List<String> timeline;

  setUp(() {
    timeline = [];
    source = FakePushSource(fidOnRegister: 'fid-1', timeline: timeline);
    api = FakeDeviceApi(timeline: timeline);
    registrar = PushRegistrar(
      source: source,
      api: api,
      deviceId: 'dev-A',
      platform: 'android',
    );
  });

  group('bật thông báo', () {
    test(
      'được phép: nghe onRegistered TRƯỚC khi register, rồi gửi lên backend',
      () async {
        final permission = await registrar.enable();
        await pumpEventQueue();

        expect(permission, PushPermission.granted);
        expect(source.calls, ['requestPermission', 'register(listener=true)']);
        expect(api.upserts.single, (
          deviceId: 'dev-A',
          pushId: 'fid-1',
          platform: 'android',
        ));
        expect(registrar.status.value, PushStatus.synced);
      },
    );

    for (final answer in [
      PushPermission.denied,
      PushPermission.deniedForever,
    ]) {
      test('$answer: không register, không gọi backend', () async {
        source.nextPermissionAnswer = answer;
        final permission = await registrar.enable();
        await pumpEventQueue();

        expect(permission, answer);
        expect(source.calls, ['requestPermission']);
        expect(api.calls, isEmpty);
        expect(registrar.status.value, PushStatus.off);
        expect(source.hasRegisteredListener, isFalse);
      });
    }

    test(
      'register lỗi thì failed, retry gọi lại register rồi gửi FID',
      () async {
        source.failRegister = true;
        await registrar.enable();
        expect(registrar.status.value, PushStatus.failed);
        expect(api.calls, isEmpty);

        source.failRegister = false;
        await registrar.retry();
        await pumpEventQueue();

        expect(
          source.calls.where((c) => c.startsWith('register')),
          hasLength(2),
        );
        expect(api.upserts.single.pushId, 'fid-1');
        expect(registrar.status.value, PushStatus.synced);
      },
    );
  });

  group('mở lại app (resume)', () {
    test('chưa hỏi quyền: không hiện hộp thoại, không register', () async {
      await registrar.resume();
      expect(source.calls, isEmpty);
      expect(registrar.status.value, PushStatus.off);
    });

    test('quyền đã có: register và gửi định danh', () async {
      source.permissionValue = PushPermission.granted;
      await registrar.resume();
      await pumpEventQueue();
      expect(source.calls, ['register(listener=true)']);
      expect(api.upserts.single.pushId, 'fid-1');
    });

    test('bị từ chối: không register', () async {
      source.permissionValue = PushPermission.denied;
      await registrar.resume();
      expect(source.calls, isEmpty);
    });
  });

  group('định danh đổi (onRegistered phát lại)', () {
    test('cùng định danh phát hai lần thì chỉ gửi một lần', () async {
      await registrar.enable();
      source.emitRegistered('fid-1');
      await pumpEventQueue();
      expect(api.calls, ['upsert:fid-1']);
    });

    test('định danh mới thì cập nhật cùng deviceId', () async {
      await registrar.enable();
      source.emitRegistered('fid-2');
      await pumpEventQueue();
      expect(api.upserts.map((u) => (u.deviceId, u.pushId)), [
        ('dev-A', 'fid-1'),
        ('dev-A', 'fid-2'),
      ]);
    });

    test('hai định danh liên tiếp được gửi đúng thứ tự', () async {
      await registrar.enable();
      source.emitRegistered('fid-2');
      source.emitRegistered('fid-3');
      await pumpEventQueue();
      expect(api.calls, ['upsert:fid-1', 'upsert:fid-2', 'upsert:fid-3']);
    });
  });

  group('backend lỗi', () {
    test('gửi lỗi thì failed, retry gửi lại đúng định danh đang chờ', () async {
      api.failUpsert = const DeviceApiException('HTTP 503');
      await registrar.enable();
      await pumpEventQueue();
      expect(registrar.status.value, PushStatus.failed);

      api.failUpsert = null;
      await registrar.retry();
      expect(api.upserts.single.pushId, 'fid-1');
      expect(registrar.status.value, PushStatus.synced);
    });

    test('retry khi không có gì chờ thì không làm gì', () async {
      await registrar.retry();
      expect(api.calls, isEmpty);
    });
  });

  group('đăng xuất', () {
    test('xoá ở backend TRƯỚC, rồi mới unregister ở FCM', () async {
      await registrar.enable();
      await pumpEventQueue();
      timeline.clear();

      final result = await registrar.signOut();

      expect(timeline, ['api:remove:dev-A', 'fcm:unregister']);
      expect(result, (serverRemoved: true, unregistered: true));
      expect(registrar.status.value, PushStatus.off);
    });

    test(
      'backend lỗi: vẫn unregister ở máy và báo chưa xoá được ở server',
      () async {
        await registrar.enable();
        await pumpEventQueue();
        api.failRemove = const DeviceApiException('HTTP 500');

        final result = await registrar.signOut();

        expect(result, (serverRemoved: false, unregistered: true));
        expect(source.calls, contains('unregister'));
      },
    );

    test('unregister lỗi: server vẫn đã xoá, kết quả nói rõ', () async {
      await registrar.enable();
      source.failUnregister = true;
      final result = await registrar.signOut();
      expect(result, (serverRemoved: true, unregistered: false));
    });

    test(
      'sau đăng xuất, onRegistered không còn được gửi lên backend',
      () async {
        await registrar.enable();
        await pumpEventQueue();
        await registrar.signOut();
        api.calls.clear();

        source.emitRegistered('fid-9');
        await pumpEventQueue();
        expect(api.calls, isEmpty);
        expect(source.hasRegisteredListener, isFalse);
      },
    );

    test(
      'đăng xuất luôn gọi xoá ở backend, kể cả khi phiên này chưa bật push',
      () async {
        await registrar.signOut();
        expect(api.calls, ['remove:dev-A']);
      },
    );

    test(
      'đăng nhập lại: đăng ký lại với định danh mới của cùng thiết bị',
      () async {
        await registrar.enable();
        await pumpEventQueue();
        await registrar.signOut();

        source.fidOnRegister = 'fid-2';
        await registrar.enable();
        await pumpEventQueue();

        expect(api.upserts.map((u) => u.pushId), ['fid-1', 'fid-2']);
        expect(registrar.status.value, PushStatus.synced);
      },
    );

    test('đang gửi dở thì đăng xuất chờ gửi xong rồi mới xoá', () async {
      api.gate = Completer<void>();
      await registrar.enable();
      await pumpEventQueue();
      expect(api.calls, ['upsert:fid-1']);

      final signOut = registrar.signOut();
      await pumpEventQueue();
      expect(api.calls, ['upsert:fid-1'], reason: 'remove phải chờ upsert');

      api.gate!.complete();
      final result = await signOut;
      expect(api.calls, ['upsert:fid-1', 'remove:dev-A']);
      expect(result.serverRemoved, isTrue);
    });
  });

  test('generateInstallationId cho 32 ký tự hex và khác nhau mỗi lần', () {
    final a = generateInstallationId();
    final b = generateInstallationId();
    expect(a, matches(RegExp(r'^[0-9a-f]{32}$')));
    expect(a, isNot(b));
  });
}
