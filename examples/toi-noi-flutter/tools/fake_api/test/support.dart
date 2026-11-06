import 'dart:convert';

import 'package:shelf/shelf.dart';

/// Kết quả một lần gọi handler: mã trạng thái, header và body đã giải JSON.
typedef Reply = ({int status, Map<String, String> headers, Object? json});

/// Gọi [handler] như một hàm, không qua socket.
Future<Reply> call(
  Handler handler,
  String method,
  String path, {
  Object? body,
  String? rawBody,
  Map<String, String> headers = const {},
}) async {
  final response = await handler(
    Request(
      method,
      Uri.parse('http://fake.test$path'),
      headers: {
        if (body != null) 'content-type': 'application/json',
        ...headers,
      },
      body: rawBody ?? (body == null ? null : jsonEncode(body)),
    ),
  );
  final text = await response.readAsString();
  return (
    status: response.statusCode,
    headers: response.headers,
    json: text.isEmpty ? null : jsonDecode(text),
  );
}

/// Đọc `json` của [reply] như một object.
Map<String, Object?> asMap(Reply reply) =>
    (reply.json as Map<String, Object?>?)!;

/// Tên trường -> thông báo trong body lỗi problem+json.
Map<String, Object?> errorsOf(Reply reply) =>
    asMap(reply)['errors']! as Map<String, Object?>;

/// Response JSON thành công với [body].
Response okJson(Object body, {int status = 200}) => Response(
  status,
  body: jsonEncode(body),
  headers: const {'content-type': 'application/json; charset=utf-8'},
);
