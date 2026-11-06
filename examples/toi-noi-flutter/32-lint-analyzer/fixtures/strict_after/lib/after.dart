import 'dart:async';
import 'dart:convert';

/// Đọc cước (đồng) từ body JSON của trip-service.
int fareOf(String body) {
  final json = jsonDecode(body) as Map<String, dynamic>;
  final fare = json['fareVnd'];
  if (fare is! int) {
    throw FormatException('fareVnd phải là số nguyên', body);
  }
  return fare;
}

/// Số cước đang chờ.
int pendingCount(List<Future<int>> fares) => fares.length;

void remember(String body) {
  final fares = <Future<int>>[];
  final done = Completer<int>();
  done.complete(fareOf(body));
  fares.add(done.future);
}
