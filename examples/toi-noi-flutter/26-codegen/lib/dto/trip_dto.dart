import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:toi_noi_api/toi_noi_api.dart'
    show DtoFormatException, TripSnapshot;
import 'package:toi_noi_core/toi_noi_core.dart';

import 'converters.dart';
import 'decode.dart';

part 'trip_dto.freezed.dart';
part 'trip_dto.g.dart';

/// Một chuyến như server mô tả, viết lại bằng `freezed` + `json_serializable`.
///
/// Cùng hợp đồng JSON với `TripDto` viết tay của `toi_noi_api`. `freezed` sinh
/// `==`, `hashCode`, `toString`, `copyWith`; `json_serializable` sinh
/// `fromJson`/`toJson`; các converter đặt lại những quy tắc chặt của bản viết
/// tay mà code sinh mặc định bỏ qua.
@freezed
abstract class TripDto with _$TripDto {
  const TripDto._();

  const factory TripDto({
    required String id,
    @ServiceKindConverter() required ServiceKind serviceKind,
    @TripStatusConverter() required TripStatus status,
    @NonNegativeIntConverter() required int distanceMeters,
    @JsonKey(name: 'fare') @WholeVndConverter() required int fareVnd,
    @VndCurrencyConverter() required String currency,
    String? driverId,

    /// Thời điểm đặt, luôn UTC.
    @UtcInstantConverter() required DateTime requestedAt,

    /// Số phiên bản của chuyến trên server, dùng làm `expectedVersion`.
    @NonNegativeIntConverter() required int version,
  }) = _TripDto;

  factory TripDto.fromJson(Object? json) =>
      decodeDto('TripDto', json, _$TripDtoFromJson);

  /// Dựng DTO từ domain, cho test và cho fixture.
  factory TripDto.fromDomain(Trip trip, {required int version}) => TripDto(
    id: trip.id,
    serviceKind: trip.serviceKind,
    status: trip.status,
    distanceMeters: (trip.distanceKm * 1000).round(),
    fareVnd: trip.fare.vnd,
    currency: 'VND',
    driverId: trip.driverId,
    requestedAt: trip.requestedAt,
    version: version,
  );

  /// Chuyển sang [Trip] của domain; payload vi phạm bất biến của [Trip] thì
  /// ném [DtoFormatException] thay vì để `ArgumentError` lọt ra ngoài.
  Trip toDomain() {
    try {
      return Trip(
        id: id,
        serviceKind: serviceKind,
        status: status,
        distanceKm: distanceMeters / 1000,
        fare: Money.fromVnd(fareVnd),
        driverId: driverId,
        requestedAt: requestedAt,
      );
    } on ArgumentError catch (e) {
      throw DtoFormatException('TripDto', e.name ?? r'$', '${e.message}');
    }
  }

  /// [toDomain] kèm [version].
  TripSnapshot toSnapshot() => TripSnapshot(toDomain(), version);
}
