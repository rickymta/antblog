import 'dart:convert';

import 'package:shelf/shelf.dart';

/// Lỗi có chủ đích của fake API, được chuyển thành `application/problem+json`.
///
/// Hình dạng body bám cách service .NET của bài 31–38 trả `ProblemDetails`:
/// `title`, `status`, `traceId` và, với lỗi nhập liệu, `errors` theo từng trường.
final class ProblemException implements Exception {
  const ProblemException(this.status, this.title, {this.errors});

  final int status;
  final String title;

  /// Tên trường -> danh sách thông báo, giống `HttpValidationProblemDetails`.
  final Map<String, List<String>>? errors;

  @override
  String toString() => 'ProblemException($status, $title)';
}

/// Khoá trong `Request.context` giữ mã theo dõi của request hiện tại.
const traceIdContextKey = 'toi_noi/traceId';

/// Dựng response lỗi dạng problem+json cho [request].
Response problemResponse(
  Request request,
  int status,
  String title, {
  Map<String, List<String>>? errors,
}) {
  final traceId = request.context[traceIdContextKey];
  return Response(
    status,
    body: jsonEncode(<String, Object?>{
      'title': title,
      'status': status,
      'errors': ?errors,
      'traceId': ?traceId,
    }),
    headers: const {'content-type': 'application/problem+json; charset=utf-8'},
  );
}

/// Dựng response JSON thành công.
Response jsonResponse(
  int status,
  Object body, {
  Map<String, String> headers = const {},
}) => Response(
  status,
  body: jsonEncode(body),
  headers: {'content-type': 'application/json; charset=utf-8', ...headers},
);
