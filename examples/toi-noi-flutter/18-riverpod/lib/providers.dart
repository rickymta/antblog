import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'quote.dart';
import 'quote_repository.dart';
import 'trip_draft.dart';

/// Chính sách thử lại của lab: chỉ lỗi tạm thời ([QuoteUnavailable]) mới
/// đáng thử, và chỉ một lần sau 1 giây. Mọi lỗi khác báo ngay cho người dùng.
///
/// Mặc định của Riverpod 3 khác hẳn: thử lại tới 10 lần với mọi lỗi không phải `Error`.
Duration? retryTransientOnce(int retryCount, Object error) =>
    error is QuoteUnavailable && retryCount < 1
    ? const Duration(seconds: 1)
    : null;

/// Cổng tới backend. Mặc định là bản giả có độ trễ để chạy thử; test và các
/// môi trường khác ghi đè provider này.
final quoteRepositoryProvider = Provider<QuoteRepository>(
  (ref) => FakeQuoteRepository(
    latency: const Duration(milliseconds: 400),
    unavailableAboveKm: 30,
  ),
);

/// Báo giá cho bản nháp hiện tại. `ref.watch(tripDraftProvider)` làm provider
/// tính lại mỗi khi khách đổi dịch vụ hoặc quãng đường; auto-dispose nên rời
/// màn hình là bỏ kết quả.
final quoteProvider = FutureProvider.autoDispose<Quote>((ref) {
  final draft = ref.watch(tripDraftProvider);
  final repository = ref.watch(quoteRepositoryProvider);
  return repository.fetchQuote(
    serviceKind: draft.serviceKind,
    distanceKm: draft.distanceKm,
  );
}, retry: retryTransientOnce);

/// Một chuyến theo mã: mỗi `id` có state riêng, và tự bỏ khi không ai xem.
final tripByIdProvider = FutureProvider.autoDispose.family<Trip, String>((
  ref,
  id,
) {
  return ref.watch(quoteRepositoryProvider).fetchTrip(id);
}, retry: retryTransientOnce);

/// Kết quả của thao tác "Đặt chuyến": `null` là chưa đặt (hoặc đã reset).
///
/// Dùng [AsyncNotifier] vì có method làm thay đổi state ([place]).
class PlaceTripNotifier extends AsyncNotifier<Trip?> {
  @override
  Trip? build() => null;

  /// Gửi [quote] lên server. Không tự thử lại: đặt chuyến không idempotent.
  Future<void> place(Quote quote) async {
    state = const AsyncLoading();
    // Trong method, đọc bằng `read`: ta chỉ dùng giá trị một lần, không cần
    // provider tính lại khi repository đổi.
    final repository = ref.read(quoteRepositoryProvider);
    final result = await AsyncValue.guard(() => repository.placeTrip(quote));
    // Provider có thể đã bị dispose trong lúc chờ (khách rời màn hình);
    // đụng vào `state` lúc đó sẽ ném lỗi.
    if (!ref.mounted) return;
    state = result;
  }
}

final placeTripProvider =
    AsyncNotifierProvider.autoDispose<PlaceTripNotifier, Trip?>(
      PlaceTripNotifier.new,
    );
