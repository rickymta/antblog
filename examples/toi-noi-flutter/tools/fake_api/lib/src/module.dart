import 'package:shelf_router/shelf_router.dart';

import 'store.dart';

/// Những thứ một module route cần: kho dữ liệu, đồng hồ và bộ sinh mã.
final class FakeApiContext {
  const FakeApiContext({
    required this.store,
    required this.clock,
    required this.newId,
  });

  final FakeApiStore store;

  /// Trả thời điểm UTC hiện tại; test thay bằng đồng hồ cố định.
  final DateTime Function() clock;

  /// Sinh mã chuyến mới.
  final String Function() newId;
}

/// Một nhóm route gắn vào router của fake API.
///
/// Đây là điểm mở rộng: bài xác thực (24) thêm module OTP và middleware kiểm
/// token, bài realtime (36) thêm module WebSocket, mà không phải sửa
/// `createFakeApiHandler`.
abstract interface class FakeApiModule {
  void mount(Router router, FakeApiContext context);
}
