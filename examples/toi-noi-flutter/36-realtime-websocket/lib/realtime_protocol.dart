import 'dart:convert';

import 'package:toi_noi_core/toi_noi_core.dart';

/// Mã đóng mà gateway dùng khi token thiếu, sai hoặc hết hạn lúc bắt tay.
/// Client không nối lại khi gặp mã này.
const closeCodeUnauthorized = 4401;

/// Một sự kiện **bền** của chuyến: có `sequence` tăng đúng 1 mỗi sự kiện.
final class TripEventEnvelope {
  const TripEventEnvelope({
    required this.tripId,
    required this.sequence,
    required this.version,
    required this.occurredAt,
    required this.status,
    required this.event,
  });

  final String tripId;

  /// Số thứ tự trong log của chuyến, bắt đầu từ 1, tăng đúng 1.
  final int sequence;

  /// `version` của chuyến sau sự kiện này, cùng thước đo với `version` của
  /// REST `GET /trips/{id}`: bài 45 dùng nó để hợp nhất REST và realtime.
  final int version;
  final DateTime occurredAt;

  /// Trạng thái của chuyến sau khi áp dụng [event].
  final TripStatus status;
  final TripEvent event;

  @override
  String toString() => 'TripEventEnvelope($tripId #$sequence ${status.name})';
}

/// Vị trí tài xế: dữ liệu **tạm thời**, không có `sequence`, bản mới thay bản cũ.
final class DriverLocation {
  const DriverLocation({
    required this.tripId,
    required this.driverId,
    required this.lat,
    required this.lng,
    required this.at,
    this.headingDegrees,
    this.speedMetersPerSecond,
  });

  final String tripId;
  final String driverId;
  final double lat;
  final double lng;

  /// Thời điểm gateway nhận vị trí này (UTC).
  final DateTime at;
  final double? headingDegrees;
  final double? speedMetersPerSecond;

  @override
  String toString() => 'DriverLocation($tripId, $lat, $lng, $at)';
}

/// Lời mời một cuốc gửi riêng cho một tài xế (tạm thời, không phát lại).
final class DriverOffer {
  const DriverOffer({
    required this.offerId,
    required this.tripId,
    required this.serviceKind,
    required this.distanceMeters,
    required this.fare,
    required this.expiresAt,
  });

  final String offerId;
  final String tripId;
  final ServiceKind serviceKind;
  final int distanceMeters;
  final Money fare;

  /// Quá hạn này thì lời mời không còn hiệu lực (UTC).
  final DateTime expiresAt;

  @override
  String toString() => 'DriverOffer($offerId, $tripId, $fare, $expiresAt)';
}

/// Frame server gửi xuống.
sealed class ServerFrame {
  const ServerFrame();
}

final class WelcomeFrame extends ServerFrame {
  const WelcomeFrame({required this.connectionId, required this.userId});

  final String connectionId;
  final String userId;
}

final class SubscribedFrame extends ServerFrame {
  const SubscribedFrame({
    required this.tripId,
    required this.latestSequence,
    required this.replayed,
    this.logEpoch,
  });

  final String tripId;
  final int latestSequence;
  final int replayed;

  /// Định danh của log mà `sequence` thuộc về; đổi khi gateway mất log.
  final String? logEpoch;
}

/// Xác nhận `subscribe` kênh `offers`.
final class OffersSubscribedFrame extends ServerFrame {
  const OffersSubscribedFrame();
}

final class OfferFrame extends ServerFrame {
  const OfferFrame(this.offer);

  final DriverOffer offer;
}

final class UnsubscribedFrame extends ServerFrame {
  const UnsubscribedFrame(this.tripId);

  final String tripId;
}

final class EventFrame extends ServerFrame {
  const EventFrame(this.envelope);

  final TripEventEnvelope envelope;
}

final class LocationFrame extends ServerFrame {
  const LocationFrame(this.location);

  final DriverLocation location;
}

final class PongFrame extends ServerFrame {
  const PongFrame();
}

final class ErrorFrame extends ServerFrame {
  const ErrorFrame({
    required this.code,
    required this.message,
    this.tripId,
    this.channel,
  });

  final String code;
  final String message;
  final String? tripId;

  /// `offers` khi lỗi thuộc kênh lời mời, không phải một chuyến.
  final String? channel;
}

/// Frame có `type` mà client này chưa biết. Bỏ qua thay vì coi là lỗi, để
/// server thêm loại frame mới mà bản app cũ không vỡ.
final class UnknownFrame extends ServerFrame {
  const UnknownFrame(this.type);

  final String type;
}

/// Đọc một text frame của server. Ném [FormatException] khi frame không phải
/// JSON object có `type` hoặc thiếu trường bắt buộc của loại đã biết.
ServerFrame parseServerFrame(String text) {
  final Object? decoded;
  try {
    decoded = jsonDecode(text);
  } on FormatException {
    throw const FormatException('Frame không phải JSON');
  }
  if (decoded is! Map<String, Object?>) {
    throw const FormatException('Frame không phải JSON object');
  }
  final type = _string(decoded, 'type');
  return switch (type) {
    'welcome' => WelcomeFrame(
      connectionId: _string(decoded, 'connectionId'),
      userId: _string(decoded, 'userId'),
    ),
    'subscribed' when decoded['channel'] == 'offers' =>
      const OffersSubscribedFrame(),
    'subscribed' => SubscribedFrame(
      tripId: _string(decoded, 'tripId'),
      latestSequence: _int(decoded, 'latestSequence'),
      replayed: _int(decoded, 'replayed'),
      logEpoch: _optionalString(decoded, 'logEpoch'),
    ),
    'offer' => OfferFrame(_offer(decoded)),
    'unsubscribed' => UnsubscribedFrame(_string(decoded, 'tripId')),
    'event' => EventFrame(_envelope(decoded)),
    'location' => LocationFrame(_location(decoded)),
    'pong' => const PongFrame(),
    'error' => ErrorFrame(
      code: _string(decoded, 'code'),
      message: _string(decoded, 'message'),
      tripId: _optionalString(decoded, 'tripId'),
      channel: _optionalString(decoded, 'channel'),
    ),
    _ => UnknownFrame(type),
  };
}

TripEventEnvelope _envelope(Map<String, Object?> json) {
  final status = TripStatus.values.asNameMap()[_string(json, 'status')];
  if (status == null) {
    throw FormatException('status lạ: ${json['status']}');
  }
  return TripEventEnvelope(
    tripId: _string(json, 'tripId'),
    sequence: _int(json, 'sequence'),
    version: _int(json, 'version'),
    occurredAt: _time(json, 'occurredAt'),
    status: status,
    event: tripEventFromJson(_object(json, 'event')),
  );
}

DriverOffer _offer(Map<String, Object?> json) {
  final kind = ServiceKind.values.asNameMap()[_string(json, 'serviceKind')];
  if (kind == null) {
    throw FormatException('serviceKind lạ: ${json['serviceKind']}');
  }
  return DriverOffer(
    offerId: _string(json, 'offerId'),
    tripId: _string(json, 'tripId'),
    serviceKind: kind,
    distanceMeters: _int(json, 'distanceMeters'),
    fare: Money.fromVnd(_int(json, 'fare')),
    expiresAt: _time(json, 'expiresAt'),
  );
}

DriverLocation _location(Map<String, Object?> json) => DriverLocation(
  tripId: _string(json, 'tripId'),
  driverId: _string(json, 'driverId'),
  lat: _double(json, 'lat'),
  lng: _double(json, 'lng'),
  at: _time(json, 'at'),
  headingDegrees: _optionalDouble(json, 'headingDegrees'),
  speedMetersPerSecond: _optionalDouble(json, 'speedMetersPerSecond'),
);

/// Đọc `event` của frame `event` thành [TripEvent] của `toi_noi_core`.
TripEvent tripEventFromJson(Map<String, Object?> json) =>
    switch (_string(json, 'type')) {
      'searchStarted' => const SearchStarted(),
      'driverAccepted' => DriverAccepted(_string(json, 'driverId')),
      'driverArrived' => const DriverArrived(),
      'pickedUp' => const PickedUp(),
      'tripStarted' => const TripStarted(),
      'completed' => const Completed(),
      'cancelled' => Cancelled(
        by:
            CancelledBy.values.asNameMap()[_string(json, 'by')] ??
            (throw FormatException('by lạ: ${json['by']}')),
      ),
      final other => throw FormatException('event lạ: $other'),
    };

/// Frame client gửi lên.
String encodeSubscribe(String tripId, int lastSequence, {String? logEpoch}) =>
    jsonEncode({
      'type': 'subscribe',
      'tripId': tripId,
      'lastSequence': lastSequence,
      'logEpoch': ?logEpoch,
    });

String encodeSubscribeOffers() =>
    jsonEncode({'type': 'subscribe', 'channel': 'offers'});

String encodeUnsubscribeOffers() =>
    jsonEncode({'type': 'unsubscribe', 'channel': 'offers'});

String encodeUnsubscribe(String tripId) =>
    jsonEncode({'type': 'unsubscribe', 'tripId': tripId});

String encodePing(String id) => jsonEncode({'type': 'ping', 'id': id});

String _string(Map<String, Object?> json, String key) {
  final value = json[key];
  if (value is String && value.isNotEmpty) return value;
  throw FormatException('Thiếu hoặc sai kiểu trường "$key"');
}

int _int(Map<String, Object?> json, String key) {
  final value = json[key];
  if (value is int && value >= 0) return value;
  throw FormatException('Thiếu hoặc sai kiểu trường "$key"');
}

double _double(Map<String, Object?> json, String key) {
  final value = json[key];
  if (value is num && value.isFinite) return value.toDouble();
  throw FormatException('Thiếu hoặc sai kiểu trường "$key"');
}

String? _optionalString(Map<String, Object?> json, String key) =>
    switch (json[key]) {
      final String value => value,
      _ => null,
    };

double? _optionalDouble(Map<String, Object?> json, String key) =>
    json[key] == null ? null : _double(json, key);

DateTime _time(Map<String, Object?> json, String key) {
  final text = _string(json, key);
  // `DateTime.tryParse` đổi "+07:00" thành UTC chứ không từ chối, nên phải
  // kiểm hậu tố `Z` mà hợp đồng quy định.
  final parsed = text.endsWith('Z') ? DateTime.tryParse(text) : null;
  if (parsed == null || !parsed.isUtc) {
    throw FormatException(
      'Trường "$key" phải là thời điểm UTC ISO-8601 (đuôi Z)',
    );
  }
  return parsed;
}

Map<String, Object?> _object(Map<String, Object?> json, String key) {
  final value = json[key];
  if (value is Map<String, Object?>) return value;
  throw FormatException('Thiếu hoặc sai kiểu trường "$key"');
}
