import 'package:shelf/shelf.dart';
import 'package:shelf_router/shelf_router.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'input.dart';
import 'module.dart';
import 'problem.dart';

/// `POST /quotes`: báo giá bằng `calculateFare` của `toi_noi_core`.
///
/// Mọi dịch vụ dùng chung một công thức (bảng giá minh hoạ).
final class QuotesModule implements FakeApiModule {
  const QuotesModule();

  @override
  void mount(Router router, FakeApiContext context) {
    router.post('/quotes', (Request request) async {
      final input = readTripInput(await readJsonObject(request));
      final fare = calculateFare(input.distanceKm);
      return jsonResponse(200, <String, Object?>{
        'serviceKind': input.serviceKind.name,
        'distanceMeters': toMeters(input.distanceKm),
        'fare': fare.vnd,
        'currency': 'VND',
      });
    });
  }
}
