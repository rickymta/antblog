import 'dart:async';

import 'package:toi_noi_api/toi_noi_api.dart';

import 'auth_api.dart';
import 'token_store.dart';

/// Không còn phiên hợp lệ: server đã từ chối refresh token, hoặc chưa từng có
/// token. Trạng thái "đã đăng xuất" được phát qua [SessionManager.signedOut],
/// còn lỗi này chỉ để dừng việc đang làm dở.
final class SessionExpired implements Exception {
  const SessionExpired();

  @override
  String toString() => 'SessionExpired';
}

/// Giữ cặp token hiện tại và là nơi **duy nhất** được đổi nó.
///
/// Hai việc nó phải đảm bảo, vì chúng quyết định app có bị đăng xuất oan hay không:
///
/// 1. **Một lần refresh cho mọi request đang chờ.** Mười request cùng nhận
///    401 chỉ gây một lời gọi `POST /auth/refresh`; chín request kia dùng
///    chung kết quả ([refreshAfterUnauthorized]). Với rotation, gửi refresh
///    token cũ lần hai bị server coi là bị đánh cắp và thu hồi cả họ token.
/// 2. **Token mới được lưu trước khi ai dùng nó**, và đăng xuất trong lúc
///    đang refresh hay đang lưu không bị "sống lại" bởi kết quả về muộn.
///
/// Token được giữ trong bộ nhớ sau lần đọc đầu, nên các phép so sánh "token
/// này còn là token hiện tại không" là đồng bộ: không có `await` nào chen giữa
/// lúc kiểm và lúc đặt [_inFlight].
class SessionManager {
  SessionManager({required this._store, required this._api, this.onStoreError});

  /// Gọi khi ghi kho thất bại (ví dụ `PlatformException`). Phiên vẫn chạy bằng
  /// cặp token trong bộ nhớ; lần mở app sau sẽ phải đăng nhập lại.
  final void Function(Object error)? onStoreError;

  final TokenStore _store;
  final AuthApi _api;
  final _signedOut = StreamController<void>.broadcast();

  Future<void>? _loading;
  AuthTokens? _tokens;
  Future<AuthTokens>? _inFlight;

  // Tăng mỗi khi phiên kết thúc hoặc bắt đầu; một lần refresh chỉ được lưu kết
  // quả nếu giá trị này không đổi từ lúc nó bắt đầu.
  int _epoch = 0;

  /// Số thứ tự của phiên hiện tại. Đổi khi đăng xuất hoặc đăng nhập lại, **không**
  /// đổi khi refresh. `AuthInterceptor` ghi nó vào từng request để biết một
  /// 401 muộn thuộc phiên cũ hay phiên này.
  int get epoch => _epoch;

  /// Phát một sự kiện mỗi khi phiên kết thúc, dù do người dùng đăng xuất hay
  /// do server từ chối refresh token.
  Stream<void> get signedOut => _signedOut.stream;

  /// Cặp token hiện tại, hoặc `null` nếu chưa đăng nhập. Chỉ đọc kho lần đầu.
  Future<AuthTokens?> read() async {
    await _load();
    return _tokens;
  }

  /// Bắt đầu phiên mới từ cặp token vừa xác minh OTP.
  Future<void> signIn(AuthTokens tokens) async {
    await _load();
    _epoch++;
    await _store.save(tokens);
    _tokens = tokens;
  }

  /// Đăng xuất. Xoá kho và bộ nhớ **trước**, rồi mới báo server thu hồi họ
  /// token: đăng xuất không được phụ thuộc vào mạng. Lỗi của lời gọi
  /// `logout` bị bỏ qua, vì phía người dùng việc đã xong.
  Future<void> signOut() async {
    await _load();
    final tokens = _tokens;
    if (tokens == null) {
      await _store.clear();
      return;
    }
    await _endSession();
    try {
      await _api.logout(tokens.refreshToken);
    } on ApiFailure {
      // Họ token ở server sẽ tự hết hạn; thu hồi sớm chỉ là thêm một lớp an toàn.
    }
  }

  /// Chờ lần refresh đang chạy (nếu có). Không bao giờ ném lỗi: request chờ
  /// ở đây chỉ muốn đợi, còn kết quả thất bại là việc của người khởi xướng.
  Future<void> waitForRefresh() async {
    final pending = _inFlight;
    if (pending == null) return;
    try {
      await pending;
    } on Object {
      // Người khởi xướng đã nhận lỗi này.
    }
  }

  /// Gọi khi một request mang [rejectedAccessToken] bị server trả 401. Trả cặp
  /// token dùng được để gửi lại request.
  ///
  /// - Đang có refresh chạy: dùng chung kết quả của nó.
  /// - Token hiện tại đã khác [rejectedAccessToken] (có refresh khác vừa xong
  ///   trong lúc 401 này đang bay về): trả luôn token hiện tại.
  /// - Còn lại: bắt đầu **một** lần refresh.
  ///
  /// Ném [SessionExpired] khi không còn phiên (không có token, hoặc server từ
  /// chối refresh token). Lỗi mạng, timeout và 5xx của chính lời gọi refresh
  /// được ném lại là [ApiFailure] và **giữ nguyên phiên**: mất sóng trong hầm
  /// không đáng bị đăng xuất.
  Future<AuthTokens> refreshAfterUnauthorized(
    String rejectedAccessToken,
  ) async {
    await _load();
    // Từ đây tới khi gán _inFlight không có await: không ai chen vào được.
    final pending = _inFlight;
    if (pending != null) return pending;
    final current = _tokens;
    if (current == null) throw const SessionExpired();
    if (current.accessToken != rejectedAccessToken) return current;
    return _inFlight = _refresh(current).whenComplete(() => _inFlight = null);
  }

  Future<AuthTokens> _refresh(AuthTokens current) async {
    final epoch = _epoch;
    final AuthTokens next;
    try {
      next = await _api.refresh(current.refreshToken);
    } on ClientFailure catch (failure) {
      if (failure.statusCode == 401 || failure.statusCode == 403) {
        // Hợp đồng của Tới Nơi: 401/403 là "refresh token không còn giá trị".
        // Server theo RFC 6749 trả 400 invalid_grant; đổi server thì đổi chỗ này.
        if (epoch == _epoch) await _endSession();
        throw const SessionExpired();
      }
      rethrow; // 429, 400...: lỗi của lời gọi, không phải của phiên.
    }
    if (epoch != _epoch) {
      // Người dùng đăng xuất (hoặc đăng nhập lại) trong lúc chờ: bỏ kết quả.
      throw const SessionExpired();
    }
    // Bộ nhớ trước, kho sau: server đã xoay token, nên cặp mới phải sống sót
    // dù ghi kho lỗi; còn lưu kho trước khi ai dùng, vì nếu app chết giữa
    // chừng mà chưa lưu thì cặp mới mất và cặp cũ không còn dùng được.
    _tokens = next;
    try {
      await _store.save(next);
    } on Object catch (error) {
      onStoreError?.call(error);
    }
    if (epoch != _epoch) {
      // Đăng xuất (hoặc đăng nhập lại) xảy ra ngay trong lúc ghi kho: lệnh ghi
      // có thể đến sau lệnh xoá. Đưa kho về đúng trạng thái hiện tại.
      await _restoreStore();
      throw const SessionExpired();
    }
    return next;
  }

  Future<void> _restoreStore() async {
    final current = _tokens;
    try {
      if (current == null) {
        await _store.clear();
      } else {
        await _store.save(current);
      }
    } on Object catch (error) {
      onStoreError?.call(error);
    }
  }

  Future<void> _endSession() async {
    _epoch++;
    _tokens = null;
    await _store.clear();
    if (!_signedOut.isClosed) _signedOut.add(null);
  }

  Future<void> _load() => _loading ??= _store.read().then((tokens) {
    _tokens = tokens;
  });

  /// Đóng luồng sự kiện. Gọi khi không còn dùng.
  Future<void> dispose() => _signedOut.close();
}
