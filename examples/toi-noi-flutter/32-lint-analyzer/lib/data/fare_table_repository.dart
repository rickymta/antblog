import 'package:toi_noi_32/domain/quote_repository.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

/// Thực thi tại chỗ bằng bảng giá của `toi_noi_core`, không gọi mạng.
class FareTableRepository implements QuoteRepository {
  const FareTableRepository();

  @override
  Future<Money> quote(double distanceKm) async => calculateFare(distanceKm);
}
