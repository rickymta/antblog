import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:toi_noi_32/domain/quote_repository.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

/// Chỗ cắm repository: composition root hoặc test ghi đè.
final quoteRepositoryProvider = Provider<QuoteRepository>(
  (ref) => throw UnimplementedError('Ghi đè quoteRepositoryProvider'),
);

/// Quãng đường khách đang chọn, tính bằng km.
class DistanceNotifier extends Notifier<double> {
  @override
  double build() => 5;

  void setKm(double km) => state = km;
}

final distanceProvider = NotifierProvider<DistanceNotifier, double>(
  DistanceNotifier.new,
);

/// Cước của quãng đường hiện tại; tính lại khi quãng đường đổi.
final quoteProvider = FutureProvider.autoDispose<Money>(
  (ref) =>
      ref.watch(quoteRepositoryProvider).quote(ref.watch(distanceProvider)),
);
