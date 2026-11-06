import 'package:clock/clock.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:toi_noi_api/toi_noi_api.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

/// Báo giá còn hiệu lực bao lâu kể từ lúc nhận (minh hoạ).
const quoteValidity = Duration(minutes: 2);

/// Báo giá kèm thời điểm hết hạn.
///
/// Giờ lấy từ `clock.now()` chứ không từ `DateTime.now()`: test thay được đồng
/// hồ mà không phải chờ hai phút thật.
final class ValidQuote {
  const ValidQuote({required this.dto, required this.expiresAt});

  final QuoteDto dto;

  /// Thời điểm UTC mà báo giá hết hiệu lực.
  final DateTime expiresAt;

  Money get fare => dto.toMoney();

  /// Hết hạn khi đã tới đúng [expiresAt] (không còn "trước" nó).
  bool isExpiredAt(DateTime now) => !now.isBefore(expiresAt);

  /// Theo đồng hồ hiện hành của zone (xem `clock` trong package `clock`).
  bool get isExpired => isExpiredAt(clock.now());

  @override
  bool operator ==(Object other) =>
      other is ValidQuote &&
      other.expiresAt == expiresAt &&
      other.dto.serviceKind == dto.serviceKind &&
      other.dto.distanceMeters == dto.distanceMeters &&
      other.dto.fareVnd == dto.fareVnd;

  @override
  int get hashCode =>
      Object.hash(expiresAt, dto.serviceKind, dto.distanceMeters, dto.fareVnd);
}

/// Khoá của family: record có `==` theo giá trị, còn `QuoteRequestDto` thì không.
typedef QuoteQuery = ({ServiceKind serviceKind, double distanceKm});

/// Client HTTP. Phải được ghi đè: app dựng client thật, test dựng client trên
/// adapter trong tiến trình.
final apiClientProvider = Provider<ToiNoiApiClient>(
  (ref) => throw UnimplementedError('ghi đè apiClientProvider'),
);

/// Báo giá cho một yêu cầu. Lỗi ra ngoài là [ApiFailure].
final quoteProvider = FutureProvider.autoDispose.family<ValidQuote, QuoteQuery>(
  (ref, query) async {
    final client = ref.watch(apiClientProvider);
    final dto = await guardApi(
      () => client.quote(
        QuoteRequestDto(
          serviceKind: query.serviceKind,
          distanceKm: query.distanceKm,
        ),
      ),
    );
    return ValidQuote(dto: dto, expiresAt: clock.now().add(quoteValidity));
  },
);
