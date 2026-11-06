import 'package:toi_noi_core/toi_noi_core.dart';

import 'errors.dart';
import 'json_reader.dart';

/// Đầu vào của báo giá và đặt chuyến: dịch vụ và quãng đường khách nhập.
///
/// Đặt chuyến dùng cùng đầu vào với báo giá để khách đặt đúng điều đã được báo.
final class QuoteRequestDto {
  const QuoteRequestDto({required this.serviceKind, required this.distanceKm});

  factory QuoteRequestDto.fromJson(Object? json) {
    final reader = JsonReader.of('QuoteRequestDto', json);
    return QuoteRequestDto(
      serviceKind: reader.enumByName('serviceKind', ServiceKind.values),
      distanceKm: reader.finiteDouble('distanceKm'),
    );
  }

  final ServiceKind serviceKind;

  /// Quãng đường theo km như khách nhập. Miền 0..100 do server quyết định.
  final double distanceKm;

  Map<String, Object?> toJson() {
    if (!distanceKm.isFinite) {
      throw ArgumentError.value(distanceKm, 'distanceKm', 'phải là số hữu hạn');
    }
    return <String, Object?>{
      'serviceKind': serviceKind.name,
      'distanceKm': distanceKm,
    };
  }
}

/// Kết quả báo giá của server.
final class QuoteDto {
  const QuoteDto({
    required this.serviceKind,
    required this.distanceMeters,
    required this.fareVnd,
  });

  factory QuoteDto.fromJson(Object? json) {
    final reader = JsonReader.of('QuoteDto', json)
      ..expectString('currency', 'VND');
    return QuoteDto(
      serviceKind: reader.enumByName('serviceKind', ServiceKind.values),
      distanceMeters: reader.nonNegativeInt('distanceMeters'),
      fareVnd: reader.wholeVnd('fare'),
    );
  }

  final ServiceKind serviceKind;

  /// Quãng đường theo mét nguyên: dạng truyền và lưu.
  final int distanceMeters;

  /// Cước theo số đồng.
  final int fareVnd;

  /// Cước dưới dạng [Money] của domain.
  Money toMoney() => Money.fromVnd(fareVnd);

  Map<String, Object?> toJson() => <String, Object?>{
    'serviceKind': serviceKind.name,
    'distanceMeters': distanceMeters,
    'fare': fareVnd,
    'currency': 'VND',
  };
}

/// Chuyến ở phía domain cùng `version` của nó trên server.
///
/// `version` không thuộc `Trip` (domain không biết gì về cách server đếm
/// phiên bản) nhưng app tài xế cần nó làm `expectedVersion` khi nhận chuyến,
/// và client cần nó để biết bản chụp nào mới hơn (bài 45). [TripDto.toSnapshot]
/// giữ hai thứ đi cùng nhau.
final class TripSnapshot {
  const TripSnapshot(this.trip, this.version);

  final Trip trip;
  final int version;

  @override
  bool operator ==(Object other) =>
      other is TripSnapshot && other.trip == trip && other.version == version;

  @override
  int get hashCode => Object.hash(trip, version);

  @override
  String toString() => 'TripSnapshot($trip, version: $version)';
}

/// Một chuyến như server mô tả.
///
/// DTO là hình dạng của dây dẫn: tên trường, kiểu và đơn vị theo hợp đồng
/// API. [toDomain] chuyển sang [Trip] của `toi_noi_core`, nơi đơn vị là km và
/// tiền là [Money].
final class TripDto {
  const TripDto({
    required this.id,
    required this.serviceKind,
    required this.status,
    required this.distanceMeters,
    required this.fareVnd,
    required this.driverId,
    required this.requestedAt,
    required this.version,
  });

  /// Dựng DTO từ domain, cho test và cho fixture.
  factory TripDto.fromDomain(Trip trip, {required int version}) => TripDto(
    id: trip.id,
    serviceKind: trip.serviceKind,
    status: trip.status,
    distanceMeters: (trip.distanceKm * 1000).round(),
    fareVnd: trip.fare.vnd,
    driverId: trip.driverId,
    requestedAt: trip.requestedAt,
    version: version,
  );

  factory TripDto.fromJson(Object? json) {
    final reader = JsonReader.of('TripDto', json)
      ..expectString('currency', 'VND');
    return TripDto(
      id: reader.string('id'),
      serviceKind: reader.enumByName('serviceKind', ServiceKind.values),
      status: reader.enumByName('status', TripStatus.values),
      distanceMeters: reader.nonNegativeInt('distanceMeters'),
      fareVnd: reader.wholeVnd('fare'),
      driverId: reader.stringOrNull('driverId'),
      requestedAt: reader.utcInstant('requestedAt'),
      version: reader.nonNegativeInt('version'),
    );
  }

  final String id;
  final ServiceKind serviceKind;
  final TripStatus status;
  final int distanceMeters;
  final int fareVnd;
  final String? driverId;

  /// Thời điểm đặt, luôn UTC.
  final DateTime requestedAt;

  /// Số phiên bản của chuyến trên server, dùng làm `expectedVersion` khi nhận.
  final int version;

  /// Chuyển sang [Trip] của domain.
  ///
  /// Nếu payload hợp lệ về kiểu nhưng vi phạm bất biến của [Trip] (ví dụ
  /// `accepted` mà không có tài xế) thì ném [DtoFormatException] thay vì để
  /// `ArgumentError` của domain lọt ra ngoài.
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

  /// [toDomain] kèm [version]; cùng quy tắc ném [DtoFormatException].
  TripSnapshot toSnapshot() => TripSnapshot(toDomain(), version);

  Map<String, Object?> toJson() => <String, Object?>{
    'id': id,
    'serviceKind': serviceKind.name,
    'status': status.name,
    'distanceMeters': distanceMeters,
    'fare': fareVnd,
    'currency': 'VND',
    'driverId': driverId,
    'requestedAt': requestedAt.toUtc().toIso8601String(),
    'version': version,
  };
}
