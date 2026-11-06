import 'dart:async';
import 'dart:convert';

/// Đọc cước (đồng) từ body JSON của trip-service.
int fareOf(String body) {
  final Map<String, dynamic> json = jsonDecode(body); // strict-casts
  final int fare = json['fareVnd']; // strict-casts
  return fare;
}

/// Số cước đang chờ.
int pendingCount(List fares) => fares.length; // strict-raw-types

void remember(String body) {
  final fares = []; // strict-inference
  final done = Completer(); // strict-inference
  done.complete(fareOf(body));
  fares.add(done.future);
}
