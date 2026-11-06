import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'quote.dart';
import 'quote_repository.dart';

/// Cổng tới backend. Mặc định là bản giả có độ trễ; test ghi đè provider này.
final quoteRepositoryProvider = Provider<QuoteRepository>(
  (ref) => FakeQuoteRepository(latency: const Duration(milliseconds: 400)),
);

/// Thứ khách đang chọn: dịch vụ và quãng đường. Bất biến, có `==`.
final class QuoteRequest {
  const QuoteRequest({required this.serviceKind, required this.distanceKm});

  final ServiceKind serviceKind;
  final double distanceKm;

  @override
  bool operator ==(Object other) =>
      other is QuoteRequest &&
      other.serviceKind == serviceKind &&
      other.distanceKm == distanceKm;

  @override
  int get hashCode => Object.hash(serviceKind, distanceKm);
}

class QuoteRequestNotifier extends Notifier<QuoteRequest> {
  @override
  QuoteRequest build() =>
      const QuoteRequest(serviceKind: ServiceKind.ride, distanceKm: 5);

  void selectService(ServiceKind kind) {
    state = QuoteRequest(serviceKind: kind, distanceKm: state.distanceKm);
  }

  void setDistanceKm(double km) {
    state = QuoteRequest(serviceKind: state.serviceKind, distanceKm: km);
  }
}

final quoteRequestProvider =
    NotifierProvider<QuoteRequestNotifier, QuoteRequest>(
      QuoteRequestNotifier.new,
    );

/// Báo giá cho yêu cầu hiện tại; tính lại mỗi khi khách đổi dịch vụ hoặc
/// quãng đường. `retry: (_, _) => null` tắt thử lại tự động: màn hình có nút
/// "Thử lại" riêng.
final quoteProvider = FutureProvider.autoDispose<Quote>((ref) {
  final request = ref.watch(quoteRequestProvider);
  final repository = ref.watch(quoteRepositoryProvider);
  return repository.fetchQuote(
    serviceKind: request.serviceKind,
    distanceKm: request.distanceKm,
  );
}, retry: (_, _) => null);
