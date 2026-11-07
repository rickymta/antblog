/// Một điểm vị trí của tài xế, gắn với đúng một chuyến.
///
/// Bất biến. `capturedAt` luôn là UTC để so sánh tuổi mẫu không phụ thuộc
/// múi giờ của máy.
final class LocationSample {
  LocationSample({
    required this.tripId,
    required this.latitude,
    required this.longitude,
    required this.accuracyMeters,
    required this.capturedAt,
  }) : assert(capturedAt.isUtc, 'capturedAt phải là giờ UTC');

  /// Đọc từ JSON đã lưu trong bộ đệm. Ném [FormatException] khi sai kiểu,
  /// thiếu khoá hoặc thời điểm không phải UTC.
  factory LocationSample.fromJson(Object? json) {
    if (json is! Map<String, Object?>) {
      throw FormatException('Mẫu vị trí phải là object, nhận được $json');
    }
    final tripId = json['tripId'];
    final lat = json['lat'];
    final lng = json['lng'];
    final accuracy = json['accuracyM'];
    final at = json['at'];
    if (tripId is! String ||
        lat is! num ||
        lng is! num ||
        accuracy is! num ||
        at is! String) {
      throw FormatException('Mẫu vị trí thiếu hoặc sai kiểu khoá: $json');
    }
    final capturedAt = DateTime.parse(at);
    if (!capturedAt.isUtc) {
      throw FormatException('Thời điểm phải là UTC (đuôi Z): $at');
    }
    return LocationSample(
      tripId: tripId,
      latitude: lat.toDouble(),
      longitude: lng.toDouble(),
      accuracyMeters: accuracy.toDouble(),
      capturedAt: capturedAt,
    );
  }

  final String tripId;
  final double latitude;
  final double longitude;
  final double accuracyMeters;
  final DateTime capturedAt;

  /// Khoá để server khử trùng khi cùng một mẫu được gửi hai lần
  /// (ví dụ app và task nền cùng đọc một bộ đệm).
  String get dedupeKey => '$tripId@${capturedAt.toIso8601String()}';

  Map<String, Object?> toJson() => {
    'tripId': tripId,
    'lat': latitude,
    'lng': longitude,
    'accuracyM': accuracyMeters,
    'at': capturedAt.toIso8601String(),
  };

  @override
  bool operator ==(Object other) =>
      other is LocationSample &&
      other.tripId == tripId &&
      other.latitude == latitude &&
      other.longitude == longitude &&
      other.accuracyMeters == accuracyMeters &&
      other.capturedAt == capturedAt;

  @override
  int get hashCode =>
      Object.hash(tripId, latitude, longitude, accuracyMeters, capturedAt);

  @override
  String toString() => 'LocationSample($dedupeKey, $latitude, $longitude)';
}
