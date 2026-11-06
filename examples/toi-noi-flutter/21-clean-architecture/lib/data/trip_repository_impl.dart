import 'package:toi_noi_21/domain/domain.dart';
import 'package:toi_noi_api/toi_noi_api.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'mappers.dart';

/// Thực thi [TripRepository] bằng `ToiNoiApiClient`.
///
/// Mỗi phương thức làm ba việc theo thứ tự: đổi domain sang DTO, gọi client,
/// đổi DTO sang domain. Lỗi nhận ra được đi qua `guardApi` rồi thành
/// [TripFailure]; lỗi không nhận ra và huỷ request đi qua theo cách `guardApi`
/// ném lại chúng (có thể là `DioException`), không bị che thành lỗi mạng.
final class TripRepositoryImpl implements TripRepository {
  TripRepositoryImpl(this._client);

  final ToiNoiApiClient _client;

  @override
  Future<Money> quote(QuoteRequest request) => _translate(() async {
    final dto = await _client.quote(request.toDto());
    return dto.toMoney();
  });

  @override
  Future<Trip> requestTrip(QuoteRequest request, {required String intentId}) =>
      _translate(() async {
        final dto = await _client.createTrip(
          request.toDto(),
          idempotencyKey: intentId,
        );
        return dto.toDomain();
      });

  @override
  Future<Trip> getTrip(String id) => _translate(() async {
    final dto = await _client.getTrip(id);
    return dto.toDomain();
  });

  Future<T> _translate<T>(Future<T> Function() call) async {
    try {
      return await guardApi(call);
    } on ApiFailure catch (failure, stackTrace) {
      Error.throwWithStackTrace(failure.toTripFailure(), stackTrace);
    }
  }
}
