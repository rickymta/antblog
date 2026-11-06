import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

part 'naive_trip_dto.freezed.dart';
part 'naive_trip_dto.g.dart';

/// Bản "mặc định" của cùng DTO: không converter, không `checked`.
///
/// CHỈ để so sánh trong test và trong bài viết. Đừng dùng cho dữ liệu từ
/// server: nó chấp nhận nhiều thứ mà `TripDto` viết tay và `TripDto` có
/// converter từ chối.
@freezed
abstract class NaiveTripDto with _$NaiveTripDto {
  @JsonSerializable(checked: false)
  const factory NaiveTripDto({
    required String id,
    required ServiceKind serviceKind,
    required TripStatus status,
    required int distanceMeters,
    @JsonKey(name: 'fare') required int fareVnd,
    required String currency,
    String? driverId,
    required DateTime requestedAt,
    required int version,
  }) = _NaiveTripDto;

  factory NaiveTripDto.fromJson(Map<String, dynamic> json) =>
      _$NaiveTripDtoFromJson(json);
}
