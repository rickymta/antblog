import 'dart:convert';
import 'dart:isolate';

import 'package:toi_noi_core/toi_noi_core.dart';

/// Sinh JSON lịch sử [count] chuyến, tất định (cùng [count] cho cùng chuỗi).
///
/// Cước tính bằng [calculateFare], không gõ tay. Số liệu là dữ liệu giả.
String generateTripsJson(int count) {
  const distances = <int>[500, 1200, 2000, 2001, 5000, 12500, 37250, 100000];
  const statuses = <TripStatus>[
    TripStatus.completed,
    TripStatus.completed,
    TripStatus.cancelled,
    TripStatus.searching,
    TripStatus.inTransit,
  ];
  final start = DateTime.utc(2026, 1, 1);
  final records = <Map<String, Object?>>[];
  for (var i = 0; i < count; i++) {
    final meters = distances[i % distances.length];
    final status = statuses[i % statuses.length];
    // `requested` và `searching` chưa có tài xế; `cancelled` ở đây là huỷ khi
    // đang tìm nên cũng chưa có.
    final hasDriver =
        status != TripStatus.searching && status != TripStatus.cancelled;
    records.add({
      'id': 'T${(i + 1).toString().padLeft(6, '0')}',
      'service': ServiceKind.values[i % ServiceKind.values.length].name,
      'status': status.name,
      'distanceMeters': meters,
      'fareVnd': calculateFare(meters / 1000).vnd,
      'driverId': hasDriver ? 'D${(i % 50).toString().padLeft(2, '0')}' : null,
      'requestedAt': start.add(Duration(minutes: i * 7)).toIso8601String(),
    });
  }
  return jsonEncode(records);
}

/// Parse JSON lịch sử thành danh sách [Trip].
///
/// Hàm thuần, không đụng `dart:ui`, nên chạy được ở isolate khác. Ném
/// [FormatException] nêu số thứ tự bản ghi khi dữ liệu sai.
List<Trip> parseTrips(String json) {
  final decoded = jsonDecode(json);
  if (decoded is! List<Object?>) {
    throw const FormatException('JSON gốc phải là một mảng');
  }
  final trips = <Trip>[];
  for (var i = 0; i < decoded.length; i++) {
    try {
      final record = decoded[i]! as Map<String, Object?>;
      trips.add(
        Trip(
          id: record['id']! as String,
          serviceKind: ServiceKind.values.byName(record['service']! as String),
          status: TripStatus.values.byName(record['status']! as String),
          distanceKm: (record['distanceMeters']! as int) / 1000,
          fare: Money.fromVnd(record['fareVnd']! as int),
          driverId: record['driverId'] as String?,
          requestedAt: DateTime.parse(record['requestedAt']! as String),
        ),
      );
    } on Object catch (e) {
      throw FormatException('bản ghi #$i không hợp lệ: $e');
    }
  }
  return trips;
}

/// Parse trên một isolate khác để main isolate không bị chặn.
///
/// Closure chỉ bắt [json] (một `String` bất biến) và gọi hàm top-level, nên
/// không kéo theo state nào khác sang isolate mới. Kết quả trả về không bị
/// sao chép vì `Isolate.run` gửi nó bằng `Isolate.exit`.
Future<List<Trip>> parseTripsInIsolate(String json) =>
    Isolate.run(() => parseTrips(json), debugName: 'parse-trips');
