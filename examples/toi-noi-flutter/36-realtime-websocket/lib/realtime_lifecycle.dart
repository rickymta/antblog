import 'package:flutter/widgets.dart';

import 'realtime_client.dart';

/// Nối vòng đời app với [RealtimeClient]: ngắt khi app đi nền, nối lại khi
/// quay về.
///
/// Khi app vào nền, hệ điều hành có thể đóng băng tiến trình hoặc cắt socket
/// mà app không được báo. Ngắt chủ động thì gateway giải phóng kết nối ngay,
/// và khi quay về client nối lại rồi `subscribe` tiếp từ `sequence` cuối.
///
/// Mặc định chỉ `paused` và `detached` mới ngắt: `inactive` và `hidden` là
/// trạng thái thoáng qua (cuộc gọi đến, bộ chuyển app), ngắt ở đó sẽ làm kết
/// nối chập chờn.
final class RealtimeLifecycleBinding with WidgetsBindingObserver {
  RealtimeLifecycleBinding(this.client, {WidgetsBinding? binding})
    : _binding = binding ?? WidgetsBinding.instance {
    _binding.addObserver(this);
  }

  final RealtimeClient client;
  final WidgetsBinding _binding;

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    switch (state) {
      case AppLifecycleState.paused || AppLifecycleState.detached:
        client.pause();
      case AppLifecycleState.resumed:
        client.resume();
      case AppLifecycleState.inactive || AppLifecycleState.hidden:
        break;
    }
  }

  /// Gỡ khỏi binding. Không dispose [client].
  void dispose() => _binding.removeObserver(this);
}
