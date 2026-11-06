import 'dart:math';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:toi_noi_21/domain/domain.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

/// Chỗ cắm repository. Presentation chỉ biết interface của domain; thực thi
/// nào đứng sau là việc của `lib/composition.dart` (app thật) hoặc của test.
///
/// Quên ghi đè thì đọc provider này ném `ProviderException` bọc
/// `UnimplementedError` ngay lần đầu, thay vì lặng lẽ chạy với một thực thi sai.
final tripRepositoryProvider = Provider<TripRepository>(
  (ref) => throw UnimplementedError(
    'Ghi đè tripRepositoryProvider ở composition root hoặc trong test',
  ),
);

/// Sinh `intentId` cho một ý định đặt chuyến.
final intentIdProvider = Provider<String Function()>((ref) {
  final random = Random.secure();
  return () =>
      'book-${DateTime.now().toUtc().microsecondsSinceEpoch}-'
      '${random.nextInt(1 << 32).toRadixString(36)}';
});

/// Use case đặt chuyến. Giữ ý định đang chờ nên sống suốt đời app.
final requestTripProvider = Provider<RequestTrip>(
  (ref) => RequestTrip(
    ref.watch(tripRepositoryProvider),
    newIntentId: ref.watch(intentIdProvider),
  ),
);

/// Điều khách đang nhập: chính là [QuoteRequest] của domain.
class QuoteRequestNotifier extends Notifier<QuoteRequest> {
  @override
  QuoteRequest build() =>
      const QuoteRequest(serviceKind: ServiceKind.ride, distanceKm: 5);

  void selectService(ServiceKind serviceKind) => state = QuoteRequest(
    serviceKind: serviceKind,
    distanceKm: state.distanceKm,
  );

  void setDistanceKm(double distanceKm) => state = QuoteRequest(
    serviceKind: state.serviceKind,
    distanceKm: distanceKm,
  );
}

final quoteRequestProvider =
    NotifierProvider<QuoteRequestNotifier, QuoteRequest>(
      QuoteRequestNotifier.new,
    );

/// Cước ước tính của yêu cầu hiện tại; `null` khi chưa báo giá.
///
/// Báo giá không có use case: quy tắc miền nằm ở `QuoteRequest`, nên chỉ còn
/// một lời gọi repository, và một lớp bọc đúng một dòng không thêm gì.
class QuoteController extends AsyncNotifier<Money?> {
  int _generation = 0;

  @override
  Money? build() {
    ref.watch(quoteRequestProvider);
    _generation++;
    return null;
  }

  Future<void> fetch() async {
    final generation = ++_generation;
    final request = ref.read(quoteRequestProvider);
    final repository = ref.read(tripRepositoryProvider);
    state = const AsyncLoading();
    final result = await AsyncValue.guard(() {
      request.requireWithinRange();
      return repository.quote(request);
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

enum _BookingOp { book, refresh }

/// Chuyến vừa đặt; `null` khi chưa đặt.
class BookingController extends AsyncNotifier<Trip?> {
  Trip? _trip;
  _BookingOp _lastOp = _BookingOp.book;

  @override
  Trip? build() => null;

  Future<void> book() async {
    if (state.isLoading) {
      return;
    }
    _lastOp = _BookingOp.book;
    final request = ref.read(quoteRequestProvider);
    final requestTrip = ref.read(requestTripProvider);
    state = const AsyncLoading();
    final result = await AsyncValue.guard<Trip?>(() => requestTrip(request));
    if (!ref.mounted) {
      return;
    }
    if (result is AsyncData<Trip?>) {
      _trip = result.value;
    }
    state = result;
  }

  /// Đọc lại chuyến hiện tại (ví dụ để thấy tài xế đã nhận).
  Future<void> refresh() async {
    final trip = _trip;
    if (trip == null || state.isLoading) {
      return;
    }
    _lastOp = _BookingOp.refresh;
    final repository = ref.read(tripRepositoryProvider);
    state = const AsyncLoading();
    final result = await AsyncValue.guard<Trip?>(
      () => repository.getTrip(trip.id),
    );
    if (!ref.mounted) {
      return;
    }
    if (result is AsyncData<Trip?>) {
      _trip = result.value;
    }
    state = result;
  }

  /// Làm lại đúng thao tác vừa thất bại.
  Future<void> retry() => _lastOp == _BookingOp.refresh ? refresh() : book();
}

final bookingControllerProvider =
    AsyncNotifierProvider<BookingController, Trip?>(BookingController.new);
