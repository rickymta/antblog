import 'package:toi_noi_core/toi_noi_core.dart';

/// Cổng lấy báo giá. Domain sở hữu interface; lớp data thực thi nó.
abstract interface class QuoteRepository {
  /// Cước cho quãng đường [distanceKm] (0 đến 100 km).
  Future<Money> quote(double distanceKm);
}
