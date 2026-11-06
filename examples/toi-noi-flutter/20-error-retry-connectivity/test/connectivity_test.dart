import 'dart:async';

import 'package:connectivity_plus_platform_interface/connectivity_plus_platform_interface.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_20/connectivity.dart';

import 'support.dart';

/// Platform giả cho `connectivity_plus`: test không cần platform channel.
final class _FakePlatform extends ConnectivityPlatform {
  List<ConnectivityResult> now = [ConnectivityResult.none];
  final controller = StreamController<List<ConnectivityResult>>.broadcast();

  @override
  Future<List<ConnectivityResult>> checkConnectivity() async => now;

  @override
  Stream<List<ConnectivityResult>> get onConnectivityChanged =>
      controller.stream;
}

void main() {
  group('linkTypeOf', () {
    test('bảng ánh xạ', () {
      final table = <List<ConnectivityResult>, LinkType>{
        [ConnectivityResult.none]: LinkType.none,
        [ConnectivityResult.wifi]: LinkType.wifi,
        [ConnectivityResult.mobile]: LinkType.cellular,
        [ConnectivityResult.ethernet]: LinkType.other,
        [ConnectivityResult.vpn]: LinkType.other,
        [ConnectivityResult.bluetooth]: LinkType.other,
        [ConnectivityResult.other]: LinkType.other,
        [ConnectivityResult.satellite]: LinkType.other,
      };
      for (final entry in table.entries) {
        expect(linkTypeOf(entry.key), entry.value, reason: '${entry.key}');
      }
    });

    test('nhiều đường: Wi-Fi thắng di động, di động thắng VPN', () {
      expect(
        linkTypeOf([ConnectivityResult.mobile, ConnectivityResult.wifi]),
        LinkType.wifi,
      );
      expect(
        linkTypeOf([ConnectivityResult.vpn, ConnectivityResult.mobile]),
        LinkType.cellular,
      );
      expect(
        linkTypeOf([ConnectivityResult.mobile, ConnectivityResult.satellite]),
        LinkType.cellular,
      );
    });
  });

  group('ConnectivityPlusMonitor trên platform giả', () {
    late _FakePlatform platform;

    setUp(() {
      platform = _FakePlatform();
      ConnectivityPlatform.instance = platform;
    });

    test('current đọc checkConnectivity rồi ánh xạ', () async {
      final monitor = ConnectivityPlusMonitor();
      expect(await monitor.current(), LinkType.none);
      platform.now = [ConnectivityResult.mobile];
      expect(await monitor.current(), LinkType.cellular);
    });

    test(
      'changes ánh xạ từng sự kiện và bỏ qua giá trị lặp liên tiếp',
      () async {
        final monitor = ConnectivityPlusMonitor();
        final seen = <LinkType>[];
        final sub = monitor.changes.listen(seen.add);
        platform.controller
          ..add([ConnectivityResult.wifi])
          ..add([ConnectivityResult.wifi]) // plugin áp distinct trên danh sách
          ..add([ConnectivityResult.none])
          ..add([ConnectivityResult.mobile, ConnectivityResult.wifi]);
        await pumpEventQueue();
        await sub.cancel();
        expect(seen, [LinkType.wifi, LinkType.none, LinkType.wifi]);
      },
    );
  });

  group('linkUpdates', () {
    test('phát giá trị hiện tại rồi các thay đổi', () async {
      final monitor = FakeConnectivity(LinkType.wifi);
      final seen = <LinkType>[];
      final sub = linkUpdates(monitor).listen(seen.add);
      await pumpEventQueue();
      monitor.emit(LinkType.none);
      monitor.emit(LinkType.cellular);
      await pumpEventQueue();
      await sub.cancel();
      expect(seen, [LinkType.wifi, LinkType.none, LinkType.cellular]);
    });

    test(
      'thay đổi đến trước câu trả lời của current: câu trả lời cũ bị bỏ',
      () async {
        final slow = Completer<LinkType>();
        final monitor = _SlowMonitor(slow.future);
        final seen = <LinkType>[];
        final sub = linkUpdates(monitor).listen(seen.add);
        await pumpEventQueue();
        monitor.controller.add(LinkType.none); // mạng vừa mất
        slow.complete(LinkType.wifi); // câu trả lời cũ, lấy từ trước đó
        await pumpEventQueue();
        await sub.cancel();
        expect(seen, [LinkType.none]);
      },
    );

    test('huỷ nghe thì huỷ cả đăng ký với monitor', () async {
      final monitor = FakeConnectivity();
      final sub = linkUpdates(monitor).listen((_) {});
      await pumpEventQueue();
      await sub.cancel();
      expect(monitor.hasListeners, isFalse);
    });
  });
}

final class _SlowMonitor implements ConnectivityMonitor {
  _SlowMonitor(this.answer);

  final Future<LinkType> answer;
  final controller = StreamController<LinkType>.broadcast();

  @override
  Future<LinkType> current() => answer;

  @override
  Stream<LinkType> get changes => controller.stream;
}
