import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

/// Phiên đăng nhập hiện tại. Bài 24 thay bằng access/refresh token thật; ở
/// đây chỉ cần biết "có hay không" và tên hiển thị.
@immutable
final class Session {
  const Session({required this.userName});

  final String userName;

  @override
  bool operator ==(Object other) =>
      other is Session && other.userName == userName;

  @override
  int get hashCode => userName.hashCode;

  @override
  String toString() => 'Session($userName)';
}

/// Nơi lưu phiên giữa các lần mở app. Bài 22 dùng `flutter_secure_storage`;
/// lab này chỉ có bản trong bộ nhớ để test không cần thiết bị.
abstract interface class SessionStore {
  Future<Session?> read();
  Future<void> write(Session session);
  Future<void> clear();
}

/// Kho không đọc được (hỏng, bị khoá). Là `Exception` nên `AuthController`
/// xử lý được; lỗi lập trình (`Error`) thì không bị nuốt.
final class SessionStoreUnavailable implements Exception {
  const SessionStoreUnavailable();

  @override
  String toString() => 'SessionStoreUnavailable';
}

/// Bản trong bộ nhớ. [latency] mô phỏng thời gian đọc kho khi khởi động,
/// [failOnRead] mô phỏng kho hỏng.
final class InMemorySessionStore implements SessionStore {
  InMemorySessionStore({
    Session? initial,
    this.latency = Duration.zero,
    this.failOnRead = false,
  }) : _session = initial;

  final Duration latency;
  final bool failOnRead;
  Session? _session;

  @override
  Future<Session?> read() async {
    await Future<void>.delayed(latency);
    if (failOnRead) throw const SessionStoreUnavailable();
    return _session;
  }

  @override
  Future<void> write(Session session) async => _session = session;

  @override
  Future<void> clear() async => _session = null;
}

final sessionStoreProvider = Provider<SessionStore>(
  (ref) => InMemorySessionStore(),
);

/// Trạng thái đăng nhập: `AsyncLoading` trong lúc đọc kho lúc khởi động,
/// sau đó `AsyncData(null)` (chưa đăng nhập) hoặc `AsyncData(session)`.
class AuthController extends AsyncNotifier<Session?> {
  @override
  Future<Session?> build() async {
    try {
      return await ref.read(sessionStoreProvider).read();
    } on Exception {
      // Kho hỏng thì coi như chưa đăng nhập; đừng để lỗi chặn mọi màn hình.
      return null;
    }
  }

  Future<void> signIn(String userName) async {
    final name = userName.trim();
    if (name.isEmpty) {
      throw ArgumentError.value(userName, 'userName', 'không được để trống');
    }
    final session = Session(userName: name);
    await ref.read(sessionStoreProvider).write(session);
    if (!ref.mounted) return;
    state = AsyncData(session);
  }

  Future<void> signOut() async {
    await ref.read(sessionStoreProvider).clear();
    if (!ref.mounted) return;
    state = const AsyncData(null);
  }
}

final authProvider = AsyncNotifierProvider<AuthController, Session?>(
  AuthController.new,
);
