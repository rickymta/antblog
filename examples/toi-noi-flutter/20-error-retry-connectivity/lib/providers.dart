import 'dart:math';

import 'package:dio/dio.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:toi_noi_api/toi_noi_api.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'connectivity.dart';
import 'failure_text.dart' show isCancellation;
import 'system_settings.dart';

/// Địa chỉ fake API. Bài 25 thay chỗ này bằng cấu hình theo flavor.
///
/// Truyền lúc chạy: `--dart-define=API_BASE_URL=http://10.0.2.2:8787`
/// (Android emulator gọi máy dev qua 10.0.2.2, iOS Simulator dùng localhost).
final apiBaseUrlProvider = Provider<Uri>(
  (ref) => Uri.parse(
    const String.fromEnvironment(
      'API_BASE_URL',
      defaultValue: 'http://localhost:8787',
    ),
  ),
);

/// Nơi nhận dòng log của `SafeLogInterceptor` và của retry; test thay bằng `list.add`.
final logSinkProvider = Provider<void Function(String line)>(
  (ref) =>
      (line) => debugPrint(line),
);

/// Adapter gửi byte. `null` là adapter thật của dio; test truyền một adapter giả.
final httpAdapterProvider = Provider<HttpClientAdapter?>((ref) => null);

/// Sinh `Idempotency-Key` cho một ý định đặt chuyến.
final idempotencyKeyProvider = Provider<String Function()>((ref) {
  final random = Random.secure();
  return () =>
      'book-${DateTime.now().toUtc().microsecondsSinceEpoch}-'
      '${random.nextInt(1 << 32).toRadixString(36)}';
});

/// Các con số của tầng mạng: timeout và chính sách thử lại. Số liệu minh hoạ.
///
/// Một lần thử có thể tốn tới `connectTimeout + receiveTimeout` (5 + 10 s).
/// Không có [maxElapsed] thì 3 lần thử cho 3 * 15 s + (0,5 s + 1 s) = 46,5 s.
/// [maxElapsed] đặt hạn tổng cho cả chuỗi thử: lần thử nào không kịp thì không
/// bắt đầu, và timeout của lần thử lại bị chặn bằng phần hạn còn lại. Hạn này
/// không với tới lần gửi đầu, vốn chỉ theo timeout của client.
class NetworkConfig {
  const NetworkConfig({
    this.connectTimeout = const Duration(seconds: 5),
    this.sendTimeout = const Duration(seconds: 10),
    this.receiveTimeout = const Duration(seconds: 10),
    this.maxAttempts = 3,
    this.baseDelay = const Duration(milliseconds: 500),
    this.maxDelay = const Duration(seconds: 4),
    this.maxElapsed = const Duration(seconds: 30),
  });

  final Duration connectTimeout;
  final Duration sendTimeout;
  final Duration receiveTimeout;
  final int maxAttempts;
  final Duration baseDelay;
  final Duration maxDelay;
  final Duration maxElapsed;
}

final networkConfigProvider = Provider<NetworkConfig>(
  (ref) => const NetworkConfig(),
);

/// Chính sách thử lại của Riverpod cho mọi provider: không thử lại.
///
/// Riverpod 3 mặc định thử lại provider ném `Exception` (tối đa 10 lần, 200 ms
/// nhân đôi tới 6,4 s). `ApiFailure` là `Exception`, nên nếu cứ để mặc định mà
/// một provider gọi API trong `build`, mỗi lần Riverpod thử lại lại kéo theo
/// đủ vòng thử của `RetryInterceptor`. Truyền hàm này cho `ProviderScope(retry:)`.
Duration? noProviderRetry(int retryCount, Object error) => null;

/// Hàm chờ của backoff. Test thay bằng hàm trả ngay hoặc dùng đồng hồ giả.
final retrySleepProvider = Provider<Sleep>((ref) => Future<void>.delayed);

/// Nguồn số ngẫu nhiên [0, 1) của jitter. Test thay bằng giá trị cố định.
final retryRandomProvider = Provider<double Function()>(
  (ref) => Random().nextDouble,
);

/// Client dùng chung: log, rồi retry (retry đứng sau log để mỗi lần thử có
/// dòng log riêng). Tạo một lần, đóng khi provider bị huỷ.
final apiClientProvider = Provider<ToiNoiApiClient>((ref) {
  final config = ref.watch(networkConfigProvider);
  final log = ref.watch(logSinkProvider);
  final client = ToiNoiApiClient(
    baseUrl: ref.watch(apiBaseUrlProvider),
    adapter: ref.watch(httpAdapterProvider),
    connectTimeout: config.connectTimeout,
    sendTimeout: config.sendTimeout,
    receiveTimeout: config.receiveTimeout,
    interceptors: [SafeLogInterceptor(log)],
  );
  client.dio.interceptors.add(
    RetryInterceptor(
      dio: client.dio,
      maxAttempts: config.maxAttempts,
      baseDelay: config.baseDelay,
      maxDelay: config.maxDelay,
      maxElapsed: config.maxElapsed,
      sleep: ref.watch(retrySleepProvider),
      random: ref.watch(retryRandomProvider),
      onRetry: (event) => log(
        'retry: lần ${event.attempt}/${config.maxAttempts} sau '
        '${event.wait.inMilliseconds} ms, vì ${event.error.type.name}',
      ),
    ),
  );
  ref.onDispose(client.close);
  return client;
});

/// Theo dõi connectivity qua interface. Mặc định dùng plugin; test thay bản giả.
final connectivityMonitorProvider = Provider<ConnectivityMonitor>(
  (ref) => ConnectivityPlusMonitor(),
);

/// Tình trạng đường truyền: giá trị hiện tại rồi mọi thay đổi.
final linkProvider = StreamProvider<LinkType>(
  (ref) => linkUpdates(ref.watch(connectivityMonitorProvider)),
);

/// Mở cài đặt hệ điều hành. Test thay bản ghi lại lời gọi.
final systemSettingsProvider = Provider<SystemSettings>(
  (ref) => const AppSettingsOpener(),
);

/// Điều khách đang nhập: dịch vụ và quãng đường theo km.
typedef QuoteForm = ({ServiceKind serviceKind, double distanceKm});

class QuoteFormNotifier extends Notifier<QuoteForm> {
  @override
  QuoteForm build() => (serviceKind: ServiceKind.ride, distanceKm: 5);

  void selectService(ServiceKind serviceKind) =>
      state = (serviceKind: serviceKind, distanceKm: state.distanceKm);

  void setDistanceKm(double distanceKm) =>
      state = (serviceKind: state.serviceKind, distanceKm: distanceKm);
}

final quoteFormProvider = NotifierProvider<QuoteFormNotifier, QuoteForm>(
  QuoteFormNotifier.new,
);

QuoteRequestDto _requestOf(QuoteForm form) =>
    QuoteRequestDto(serviceKind: form.serviceKind, distanceKm: form.distanceKm);

/// Cước ước tính của form hiện tại; `null` khi chưa báo giá.
///
/// Mọi lỗi ra khỏi `guardApi` là một [ApiFailure]: `AsyncError.error` luôn
/// switch được trên năm loại. Lỗi lập trình (không phải [ApiFailure]) vẫn
/// thành `AsyncError` nhưng mang kiểu gốc của nó. Request bị huỷ (đổi form,
/// bấm Báo giá lần nữa, đóng provider) không bao giờ thành lỗi.
class QuoteController extends AsyncNotifier<Money?> {
  int _generation = 0;
  CancelToken? _token;

  @override
  Money? build() {
    ref.watch(quoteFormProvider);
    _generation++;
    _token?.cancel('form đã đổi');
    _token = null;
    ref.onDispose(() => _token?.cancel('provider bị huỷ'));
    return null;
  }

  Future<void> fetch() async {
    final generation = ++_generation;
    final request = _requestOf(ref.read(quoteFormProvider));
    final client = ref.read(apiClientProvider);
    _token?.cancel('báo giá mới thay thế');
    final token = _token = CancelToken();
    state = const AsyncLoading();
    final result = await AsyncValue.guard(
      () => guardApi(
        () async => (await client.quote(request, cancelToken: token)).toMoney(),
      ),
    );
    if (!ref.mounted || generation != _generation) {
      return;
    }
    final error = result.error;
    if (error != null && isCancellation(error)) {
      state = const AsyncData(null);
      return;
    }
    state = result;
  }
}

final quoteControllerProvider = AsyncNotifierProvider<QuoteController, Money?>(
  QuoteController.new,
);

enum _BookingOp { book, refresh }

/// Chuyến vừa đặt; `null` khi chưa đặt.
class BookingController extends AsyncNotifier<Trip?> {
  // Khoá gắn với đúng nội dung đã gửi (xem bài 19): bấm lại sau lỗi dùng lại
  // khoá cũ, đổi nội dung thì khoá mới. RetryInterceptor cũng gửi lại với
  // đúng khoá này, nên một lần đặt chỉ ra một chuyến dù lỗi xảy ra ở đâu.
  ({String key, QuoteForm form})? _attempt;
  Trip? _trip;
  _BookingOp _lastOp = _BookingOp.book;
  CancelToken _token = CancelToken();

  @override
  Trip? build() {
    // Mỗi lần dựng có token riêng, huỷ khi provider bị bỏ: request đang bay
    // không còn ai chờ. Server có thể đã xử lý; khoá idempotency lo phần đó.
    final token = _token = CancelToken();
    ref.onDispose(() => token.cancel('provider bị huỷ'));
    return null;
  }

  Future<void> book() async {
    if (state.isLoading) {
      return;
    }
    _lastOp = _BookingOp.book;
    final form = ref.read(quoteFormProvider);
    var attempt = _attempt;
    if (attempt == null || attempt.form != form) {
      attempt = (key: ref.read(idempotencyKeyProvider)(), form: form);
      _attempt = attempt;
    }
    final client = ref.read(apiClientProvider);
    final key = attempt.key;
    final token = _token;
    state = const AsyncLoading();
    final result = await AsyncValue.guard(
      () => guardApi(() async {
        final dto = await client.createTrip(
          _requestOf(form),
          idempotencyKey: key,
          cancelToken: token,
        );
        return dto.toDomain();
      }),
    );
    if (!ref.mounted) {
      return;
    }
    if (result is AsyncData<Trip?>) {
      _attempt = null;
      _trip = result.value;
    }
    state = result;
  }

  /// Đọc lại chuyến hiện tại từ server (ví dụ để thấy tài xế đã nhận).
  Future<void> refresh() async {
    final trip = _trip;
    if (trip == null || state.isLoading) {
      return;
    }
    _lastOp = _BookingOp.refresh;
    final client = ref.read(apiClientProvider);
    final token = _token;
    state = const AsyncLoading();
    final result = await AsyncValue.guard(
      () => guardApi(
        () async =>
            (await client.getTrip(trip.id, cancelToken: token)).toDomain(),
      ),
    );
    if (!ref.mounted) {
      return;
    }
    if (result is AsyncData<Trip?>) {
      _trip = result.value;
    }
    state = result;
  }

  /// Làm lại đúng thao tác vừa thất bại: đặt chuyến hoặc làm mới.
  Future<void> retry() => _lastOp == _BookingOp.refresh ? refresh() : book();
}

final bookingControllerProvider =
    AsyncNotifierProvider<BookingController, Trip?>(BookingController.new);
