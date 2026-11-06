import 'dart:convert';

import 'package:shelf/shelf.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'problem.dart';

/// Đọc body của [request] thành một JSON object.
///
/// Body rỗng, không phải JSON hoặc không phải object ném [ProblemException]
/// 400; lỗi từng trường thuộc về 422 và do hàm gọi quyết định.
Future<Map<String, Object?>> readJsonObject(Request request) async {
  final text = await request.readAsString();
  final Object? decoded;
  try {
    decoded = jsonDecode(text);
  } on FormatException {
    throw const ProblemException(
      400,
      'Invalid request',
      errors: {
        'body': ['Expected a JSON object'],
      },
    );
  }
  if (decoded is Map<String, Object?>) {
    return decoded;
  }
  throw const ProblemException(
    400,
    'Invalid request',
    errors: {
      'body': ['Expected a JSON object'],
    },
  );
}

/// Đầu vào chung của báo giá và đặt chuyến.
typedef TripInput = ({ServiceKind serviceKind, double distanceKm});

/// Kiểm `serviceKind` và `distanceKm`; mọi lỗi gom vào một [ProblemException]
/// 422 để client thấy đủ các trường sai trong một lần.
TripInput readTripInput(Map<String, Object?> body) {
  final errors = <String, List<String>>{};

  ServiceKind? serviceKind;
  final rawKind = body['serviceKind'];
  if (rawKind is String) {
    serviceKind = ServiceKind.values.asNameMap()[rawKind];
  }
  if (serviceKind == null) {
    errors['serviceKind'] = ['Expected one of: ride, food, parcel'];
  }

  double? distanceKm;
  final rawKm = body['distanceKm'];
  if (rawKm is! num) {
    errors['distanceKm'] = ['Expected a number'];
  } else if (!(rawKm >= 0 && rawKm <= FarePolicy.maxKm)) {
    // Viết dạng "điều kiện hợp lệ" rồi phủ định để NaN cũng bị chặn.
    errors['distanceKm'] = ['Expected 0..${FarePolicy.maxKm} km'];
  } else {
    distanceKm = rawKm.toDouble();
  }

  if (errors.isNotEmpty || serviceKind == null || distanceKm == null) {
    throw ProblemException(422, 'Unprocessable request', errors: errors);
  }
  return (serviceKind: serviceKind, distanceKm: distanceKm);
}

/// Số mét nguyên của [distanceKm], cùng cách làm tròn với `calculateFare`.
int toMeters(double distanceKm) => (distanceKm * 1000).round();
