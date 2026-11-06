import 'package:dio/dio.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:toi_noi_api/toi_noi_api.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'config/app_config.dart';

/// Cấu hình của bản build. Không có giá trị mặc định có chủ ý: `main()` phải
/// ghi đè bằng `AppConfig.fromEnvironment()`, test ghi đè bằng cấu hình giả.
/// Quên ghi đè thì mọi provider phụ thuộc nổ ngay, thay vì chạy với địa chỉ
/// đoán mò.
final appConfigProvider = Provider<AppConfig>(
  (ref) => throw StateError(
    'appConfigProvider chưa được ghi đè: thêm '
    'appConfigProvider.overrideWithValue(config) vào ProviderScope.',
  ),
);

/// Adapter gửi byte. `null` là adapter thật của dio; test truyền
/// `InProcessAdapter`.
final httpAdapterProvider = Provider<HttpClientAdapter?>((ref) => null);

/// Nơi nhận dòng log HTTP; test thay bằng `list.add`.
final logSinkProvider = Provider<void Function(String line)>(
  (ref) =>
      (line) => debugPrint(line),
);

/// Client dùng chung. Địa chỉ, timeout và việc có log hay không đều lấy từ
/// [appConfigProvider], không có `if (kDebugMode)` rải rác.
final apiClientProvider = Provider<ToiNoiApiClient>((ref) {
  final config = ref.watch(appConfigProvider);
  final client = ToiNoiApiClient(
    baseUrl: config.apiBaseUrl,
    adapter: ref.watch(httpAdapterProvider),
    interceptors: [
      if (config.logHttp) SafeLogInterceptor(ref.watch(logSinkProvider)),
    ],
    connectTimeout: config.connectTimeout,
    receiveTimeout: config.receiveTimeout,
  );
  ref.onDispose(client.close);
  return client;
});

/// Báo giá thử một chuyến đi 5 km để thấy app đang nói chuyện với máy chủ nào.
class SampleQuoteController extends AsyncNotifier<Money?> {
  @override
  Money? build() => null;

  Future<void> fetch() async {
    final client = ref.read(apiClientProvider);
    state = const AsyncLoading();
    final result = await AsyncValue.guard(() async {
      final dto = await client.quote(
        const QuoteRequestDto(serviceKind: ServiceKind.ride, distanceKm: 5),
      );
      return dto.toMoney();
    });
    if (!ref.mounted) {
      return;
    }
    state = result;
  }
}

final sampleQuoteProvider =
    AsyncNotifierProvider<SampleQuoteController, Money?>(
      SampleQuoteController.new,
    );
