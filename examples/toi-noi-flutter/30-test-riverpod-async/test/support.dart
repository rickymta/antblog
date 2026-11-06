import 'dart:async';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_riverpod/misc.dart' show Override, ProviderListenable;
import 'package:toi_noi_30/driver_position.dart';
import 'package:toi_noi_30/quote_providers.dart';
import 'package:toi_noi_api/fault_injection.dart';
import 'package:toi_noi_api/toi_noi_api.dart';
import 'package:toi_noi_core/toi_noi_core.dart';
import 'package:toi_noi_fake_api/testing.dart';
import 'package:toi_noi_fake_api/toi_noi_fake_api.dart';

/// Mốc thời gian cố định (UTC) để test không phụ thuộc đồng hồ máy.
final fixedNow = DateTime.utc(2026, 11, 6, 3);

/// Yêu cầu báo giá 5 km: 25.500 VND theo bảng giá minh hoạ.
const ride5km = (serviceKind: ServiceKind.ride, distanceKm: 5.0);

/// Ngoài miền 0 đến 100 km: server trả 422.
const ride150km = (serviceKind: ServiceKind.ride, distanceKm: 150.0);

/// Fake API chạy trong tiến trình, bọc bởi [FaultyAdapter] để tiêm lỗi mạng
/// theo kịch bản. Không socket, không chờ thật.
final class ApiHarness {
  ApiHarness({List<NetworkFault?> faults = const []})
    : _server = InProcessAdapter(createFakeApiHandler()) {
    faulty = FaultyAdapter(_server, script: faults);
    client = ToiNoiApiClient(
      baseUrl: Uri.parse('http://fake.test'),
      adapter: faulty,
    );
  }

  final InProcessAdapter _server;
  late final FaultyAdapter faulty;
  late final ToiNoiApiClient client;

  /// Số lần client gọi adapter, kể cả lần bị tiêm lỗi.
  int get attempts => faulty.calls;

  /// Số request thật sự tới được fake API.
  int get serverRequests => _server.requests.length;

  Override get override => apiClientProvider.overrideWithValue(client);
}

/// Ghi lại mọi state của [provider] kể từ lúc nghe, kể cả state đầu tiên.
///
/// `container.listen` giữ provider sống, nên autoDispose không xoá state giữa
/// chừng. Mỗi phần tử là một lần listener được gọi.
List<AsyncValue<T>> record<T>(
  ProviderContainer container,
  ProviderListenable<AsyncValue<T>> provider,
) {
  final states = <AsyncValue<T>>[];
  container.listen(
    provider,
    (_, next) => states.add(next),
    fireImmediately: true,
  );
  return states;
}

/// Đổi chuỗi state của provider thành `Stream`, để dùng `emitsInOrder`.
Stream<AsyncValue<T>> stateStream<T>(
  ProviderContainer container,
  ProviderListenable<AsyncValue<T>> provider,
) {
  final controller = StreamController<AsyncValue<T>>();
  final subscription = container.listen(
    provider,
    (_, next) => controller.add(next),
    fireImmediately: true,
  );
  controller.onCancel = subscription.close;
  return controller.stream;
}

/// Vị trí tài xế mẫu.
DriverPosition position(double lat, [double lng = 106.7]) =>
    DriverPosition(lat: lat, lng: lng, at: fixedNow);

/// Nguồn vị trí giả do test điều khiển bằng `StreamController`.
///
/// Đếm số lần được nghe và số lần bị huỷ, để test kiểm tra provider có huỷ
/// subscription khi bị bỏ không.
final class FakeLocationSource implements DriverLocationSource {
  /// [broadcast] cho phép nhiều người nghe cùng lúc (nhiều tài xế dùng chung
  /// một controller); mặc định là một người nghe, chặt hơn.
  FakeLocationSource({bool broadcast = false})
    : controller = broadcast
          ? StreamController<DriverPosition>.broadcast()
          : StreamController<DriverPosition>() {
    controller.onCancel = () => cancels++;
  }

  final StreamController<DriverPosition> controller;
  final List<String> watched = [];
  int cancels = 0;

  bool get hasListener => controller.hasListener;

  void emit(DriverPosition value) => controller.add(value);

  void fail(Object error) => controller.addError(error);

  @override
  Stream<DriverPosition> watch(String driverId) {
    watched.add(driverId);
    return controller.stream;
  }
}
