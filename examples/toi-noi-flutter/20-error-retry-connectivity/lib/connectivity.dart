import 'dart:async';

import 'package:connectivity_plus/connectivity_plus.dart';

/// Loại đường truyền mà thiết bị báo đang có.
///
/// Đây chỉ là tình trạng của "ống", không phải bằng chứng gọi được server:
/// Wi-Fi của quán cà phê chưa đăng nhập vẫn báo [wifi].
enum LinkType { none, wifi, cellular, other }

/// Gộp danh sách `ConnectivityResult` của `connectivity_plus` thành một [LinkType].
///
/// Thứ tự ưu tiên: Wi-Fi, di động, rồi mọi thứ khác (ethernet, VPN, bluetooth,
/// vệ tinh, other). Theo README của plugin, [ConnectivityResult.none] chỉ xuất
/// hiện một mình, và Android khi cả Wi-Fi lẫn di động bật chỉ báo Wi-Fi.
LinkType linkTypeOf(List<ConnectivityResult> results) {
  if (!results.hasConnectivity) {
    return LinkType.none;
  }
  if (results.contains(ConnectivityResult.wifi)) {
    return LinkType.wifi;
  }
  if (results.contains(ConnectivityResult.mobile)) {
    return LinkType.cellular;
  }
  return LinkType.other;
}

/// Ranh giới với `connectivity_plus`: phần còn lại của app chỉ biết interface này,
/// nên test thay bằng bản giả mà không cần platform channel.
abstract interface class ConnectivityMonitor {
  /// Tình trạng hiện tại.
  Future<LinkType> current();

  /// Mỗi lần tình trạng đổi. Với plugin, iOS có thể phát "none" rồi "wifi" liền
  /// sau khi nối lại, nên đừng coi mỗi sự kiện là một biến cố thật.
  Stream<LinkType> get changes;
}

/// Cài đặt thật dựa trên `connectivity_plus`.
final class ConnectivityPlusMonitor implements ConnectivityMonitor {
  ConnectivityPlusMonitor([Connectivity? connectivity])
    : _connectivity = connectivity ?? Connectivity();

  final Connectivity _connectivity;

  @override
  Future<LinkType> current() async =>
      linkTypeOf(await _connectivity.checkConnectivity());

  @override
  Stream<LinkType> get changes =>
      _connectivity.onConnectivityChanged.map(linkTypeOf);
}

/// Giá trị hiện tại rồi mọi thay đổi, thành một luồng duy nhất.
///
/// Đăng ký nghe [ConnectivityMonitor.changes] **trước** khi hỏi
/// [ConnectivityMonitor.current]: viết ngược lại thì một thay đổi xảy ra giữa
/// hai bước bị mất. Nếu có thay đổi đến trước khi câu trả lời của `current`
/// về tới, câu trả lời đó đã cũ và bị bỏ.
Stream<LinkType> linkUpdates(ConnectivityMonitor monitor) {
  late final StreamController<LinkType> controller;
  StreamSubscription<LinkType>? subscription;
  controller = StreamController<LinkType>(
    onListen: () {
      var changed = false;
      subscription = monitor.changes.listen((link) {
        changed = true;
        controller.add(link);
      }, onError: controller.addError);
      monitor.current().then((link) {
        if (!changed && !controller.isClosed) {
          controller.add(link);
        }
      }, onError: controller.addError);
    },
    onCancel: () => subscription?.cancel(),
  );
  return controller.stream;
}
