import 'dart:math';

import 'package:dio/dio.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:toi_noi_api/toi_noi_api.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

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

/// Nơi nhận dòng log của `SafeLogInterceptor`; test thay bằng `list.add`.
///
/// `debugPrint` vẫn in ra ở bản release, nên chỉ ghi khi `kDebugMode`. Dòng log
/// đã che header, query và body, nhưng đường dẫn có mã chuyến.
final logSinkProvider = Provider<void Function(String line)>(
  (ref) => (line) {
    if (kDebugMode) {
      debugPrint(line);
    }
  },
);

/// Adapter gửi byte. `null` là adapter thật của dio; test truyền
/// `InProcessAdapter` để chạy fake API ngay trong tiến trình.
final httpAdapterProvider = Provider<HttpClientAdapter?>((ref) => null);

/// Sinh `Idempotency-Key` cho một ý định đặt chuyến.
final idempotencyKeyProvider = Provider<String Function()>((ref) {
  final random = Random.secure();
  return () =>
      'book-${DateTime.now().toUtc().microsecondsSinceEpoch}-'
      '${random.nextInt(1 << 32).toRadixString(36)}';
});

/// Client dùng chung. Tạo một lần, đóng khi provider bị huỷ.
final apiClientProvider = Provider<ToiNoiApiClient>((ref) {
  final client = ToiNoiApiClient(
    baseUrl: ref.watch(apiBaseUrlProvider),
    adapter: ref.watch(httpAdapterProvider),
    interceptors: [SafeLogInterceptor(ref.watch(logSinkProvider))],
  );
  ref.onDispose(client.close);
  return client;
});

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
/// `build` theo dõi form: đổi dịch vụ hay quãng đường thì báo giá cũ bị bỏ,
/// và response của request đang bay (nếu có) cũng bị bỏ qua nhờ [_generation].
class QuoteController extends AsyncNotifier<Money?> {
  int _generation = 0;

  @override
  Money? build() {
    ref.watch(quoteFormProvider);
    _generation++;
    return null;
  }

  Future<void> fetch() async {
    final generation = ++_generation;
    final request = _requestOf(ref.read(quoteFormProvider));
    final client = ref.read(apiClientProvider);
    state = const AsyncLoading();
    final result = await AsyncValue.guard(() async {
      final dto = await client.quote(request);
      return dto.toMoney();
    });
    if (!ref.mounted || generation != _generation) {
      return;
    }
    state = result;
  }
}

final quoteControllerProvider = AsyncNotifierProvider<QuoteController, Money?>(
  QuoteController.new,
);

/// Chuyến vừa đặt; `null` khi chưa đặt.
class BookingController extends AsyncNotifier<Trip?> {
  // Khoá gắn với đúng nội dung đã gửi: lần bấm lại sau khi lỗi dùng lại khoá
  // cũ (server nhận ra đây là cùng một lần đặt), còn đổi nội dung thì phải
  // là một lần đặt mới với khoá mới.
  ({String key, QuoteForm form})? _attempt;

  @override
  Trip? build() => null;

  Future<void> book() async {
    if (state.isLoading) {
      return;
    }
    if (state.value != null) {
      // Đã có chuyến: muốn đặt chuyến khác thì gọi reset() trước.
      return;
    }
    final form = ref.read(quoteFormProvider);
    var attempt = _attempt;
    if (attempt == null || attempt.form != form) {
      attempt = (key: ref.read(idempotencyKeyProvider)(), form: form);
      _attempt = attempt;
    }
    final client = ref.read(apiClientProvider);
    final key = attempt.key;
    state = const AsyncLoading();
    final result = await AsyncValue.guard(() async {
      final dto = await client.createTrip(
        _requestOf(form),
        idempotencyKey: key,
      );
      return dto.toDomain();
    });
    if (!ref.mounted) {
      return;
    }
    if (result is AsyncData<Trip?>) {
      _attempt = null;
    }
    state = result;
  }

  /// Bỏ chuyến hiện tại để màn hình đặt chuyến khác. Không huỷ gì trên server.
  void reset() {
    if (state.isLoading) {
      return;
    }
    _attempt = null;
    state = const AsyncData(null);
  }

  /// Đọc lại chuyến hiện tại từ server (ví dụ để thấy tài xế đã nhận).
  ///
  /// Trong lúc chờ và khi lỗi, `state.value` vẫn là chuyến cũ: Riverpod 3 tự
  /// chép giá trị trước sang `AsyncLoading` và `AsyncError` khi ta gán `state`.
  /// Nhờ vậy màn hình giữ được thẻ chuyến và nút thử lại.
  Future<void> refresh() async {
    final trip = state.value;
    if (trip == null || state.isLoading) {
      return;
    }
    final client = ref.read(apiClientProvider);
    state = const AsyncLoading();
    final result = await AsyncValue.guard(() async {
      final dto = await client.getTrip(trip.id);
      return dto.toDomain();
    });
    if (!ref.mounted) {
      return;
    }
    state = result;
  }
}

final bookingControllerProvider =
    AsyncNotifierProvider<BookingController, Trip?>(BookingController.new);
