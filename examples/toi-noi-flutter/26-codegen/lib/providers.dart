import 'package:riverpod_annotation/riverpod_annotation.dart';
import 'package:toi_noi_api/toi_noi_api.dart' show ApiFailure, TripSnapshot;

import 'quote_source.dart';
import 'quote_state.dart';

part 'providers.g.dart';

/// Cổng tới backend. Cố ý không có giá trị mặc định: `main()` và test ghi đè
/// bằng `quoteSourceProvider.overrideWithValue(...)`.
@Riverpod(keepAlive: true)
QuoteSource quoteSource(Ref ref) => throw StateError(
  'quoteSourceProvider chưa được ghi đè: thêm '
  'quoteSourceProvider.overrideWithValue(source) vào ProviderScope.',
);

/// State của màn hình báo giá. Không có `keepAlive` nên là auto-dispose:
/// rời màn hình thì state bị bỏ.
@riverpod
class QuoteController extends _$QuoteController {
  @override
  QuoteScreenState build() => const QuoteScreenState.idle();

  /// Hỏi giá cho [input]. Lỗi đã phân loại thành [ApiFailure] nằm trong state.
  /// Lỗi khác (một bug) vẫn ném ra để không bị che, nhưng state quay về `idle`
  /// để màn hình không kẹt ở trạng thái đang tải.
  Future<void> fetch(QuoteInput input) async {
    state = QuoteScreenState.loading(input: input);
    final source = ref.read(quoteSourceProvider);
    try {
      final quote = await source.quote(
        serviceKind: input.serviceKind,
        distanceKm: input.distanceKm,
      );
      if (!ref.mounted) return;
      state = QuoteScreenState.ready(input: input, fare: quote.toMoney());
    } on ApiFailure catch (failure) {
      if (!ref.mounted) return;
      state = QuoteScreenState.failed(input: input, failure: failure);
    } catch (_) {
      if (ref.mounted) state = const QuoteScreenState.idle();
      rethrow;
    }
  }
}

/// Một chuyến theo mã. Tham số của hàm (`id`) trở thành tham số của provider:
/// `tripSnapshotProvider('T-1')` và `tripSnapshotProvider('T-2')` có state
/// riêng (family).
@riverpod
Future<TripSnapshot> tripSnapshot(Ref ref, String id) async {
  final dto = await ref.watch(quoteSourceProvider).trip(id);
  return dto.toSnapshot();
}
