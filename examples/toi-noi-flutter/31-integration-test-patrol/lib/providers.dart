import 'package:dio/dio.dart' show HttpClientAdapter;
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:toi_noi_api/toi_noi_api.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'api_base_url.dart';
import 'notification_permission.dart';

/// Địa chỉ API của bản build này (xem [resolveApiBaseUrl]).
final apiBaseUrlProvider = Provider<Uri>((ref) => resolveApiBaseUrl());

/// Adapter gửi byte. `null` là adapter thật của dio; `flutter test` truyền
/// `InProcessAdapter` để không có mạng.
final httpAdapterProvider = Provider<HttpClientAdapter?>((ref) => null);

final apiClientProvider = Provider<ToiNoiApiClient>((ref) {
  final client = ToiNoiApiClient(
    baseUrl: ref.watch(apiBaseUrlProvider),
    adapter: ref.watch(httpAdapterProvider),
  );
  ref.onDispose(client.close);
  return client;
});

final notificationPermissionProvider = Provider<NotificationPermissionGateway>(
  (ref) => const SystemNotificationPermission(),
);

/// Khoảng cách giữa hai lần hỏi trạng thái chuyến.
final pollIntervalProvider = Provider<Duration>(
  (ref) => const Duration(seconds: 2),
);

var _keySequence = 0;

/// Tạo `Idempotency-Key` cho mỗi lần đặt chuyến.
final idempotencyKeyProvider = Provider<String Function()>(
  (ref) =>
      () => 'trip-${DateTime.now().microsecondsSinceEpoch}-${++_keySequence}',
);

/// Chuyến còn đang chờ tài xế thì app tiếp tục hỏi.
bool isWaitingForDriver(TripStatus status) =>
    status == TripStatus.requested || status == TripStatus.searching;

/// Hỏi `GET /trips/{id}` mỗi [pollIntervalProvider] cho tới khi có tài xế.
///
/// Không tự thử lại khi lỗi: để màn hình hiện lỗi và để người dùng quyết định.
final tripProvider = StreamProvider.autoDispose.family<TripSnapshot, String>((
  ref,
  id,
) async* {
  final client = ref.watch(apiClientProvider);
  final interval = ref.watch(pollIntervalProvider);
  while (ref.mounted) {
    final snapshot = (await client.getTrip(id)).toSnapshot();
    if (!ref.mounted) {
      return;
    }
    yield snapshot;
    if (!isWaitingForDriver(snapshot.trip.status)) {
      return;
    }
    await Future<void>.delayed(interval);
  }
}, retry: (retryCount, error) => null);

/// Quyền thông báo, đọc khi mở màn hình và đổi khi người dùng trả lời.
final notificationProvider =
    AsyncNotifierProvider.autoDispose<
      NotificationController,
      NotificationPermissionStatus
    >(NotificationController.new);

class NotificationController
    extends AsyncNotifier<NotificationPermissionStatus> {
  @override
  Future<NotificationPermissionStatus> build() =>
      ref.read(notificationPermissionProvider).status();

  Future<void> request() async {
    final result = await ref.read(notificationPermissionProvider).request();
    if (ref.mounted) {
      state = AsyncData(result);
    }
  }
}
