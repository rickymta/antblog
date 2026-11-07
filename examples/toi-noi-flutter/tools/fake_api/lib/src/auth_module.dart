import 'dart:convert';
import 'dart:math';

import 'package:shelf/shelf.dart';
import 'package:shelf_router/shelf_router.dart';

import 'input.dart';
import 'module.dart';
import 'problem.dart';

/// Mã OTP cố định của fake: **chỉ dành cho fake**.
///
/// Server thật sinh mã ngẫu nhiên cho từng yêu cầu, gửi qua SMS và không bao
/// giờ có một mã dùng chung. Ở đây mã cố định để test và lab chạy lặp lại
/// được; nó không nằm trong response nào.
const fakeOtpCode = '123456';

final _phonePattern = RegExp(r'^\+[0-9]{8,15}$');
final _bearerPattern = RegExp(r'^Bearer (\S+)$', caseSensitive: false);

final class _Challenge {
  _Challenge(this.phone, this.role, this.expiresAt);

  final String phone;
  final String role;
  final DateTime expiresAt;
  int wrongAttempts = 0;
}

/// Một "họ" token: mọi refresh token sinh ra từ một lần đăng nhập OTP.
final class _Family {
  _Family(this.userId, this.role);

  final String userId;
  final String role;
  bool revoked = false;
}

final class _RefreshRecord {
  _RefreshRecord(this.family, this.expiresAt);

  final _Family family;
  final DateTime expiresAt;
  bool used = false;
}

final class _AccessRecord {
  const _AccessRecord(this.userId, this.role, this.expiresAt);

  final String userId;
  final String role;
  final DateTime expiresAt;
}

/// Xác thực giả: `POST /auth/otp/request`, `POST /auth/otp/verify`,
/// `POST /auth/refresh` và `POST /auth/logout`.
///
/// **Không dùng ngoài lab.** Những điều fake này cố ý làm khác thật:
///
/// - OTP là hằng [fakeOtpCode] cho mọi yêu cầu.
/// - Access token có dạng JWT (`header.payload.chữ-ký`) nhưng **không ký**:
///   header ghi `alg: none` và phần chữ ký để trống. Server fake nhận token
///   bằng cách tra nguyên chuỗi trong bảng đã phát, không đọc `payload`, nên
///   sửa một ký tự là token bị từ chối. Client **không được** coi payload là
///   đáng tin: ai cũng sửa được nó.
/// - Refresh token là chuỗi ngẫu nhiên đục (không phải JWT).
/// - Mọi trạng thái nằm trong bộ nhớ của đối tượng này, mất khi tắt server.
///
/// **Xoay vòng refresh token (rotation).** Mỗi lần `refresh` thành công, token
/// cũ bị đánh dấu đã dùng và một cặp mới ra đời trong cùng một họ. Đưa lại một
/// refresh token đã dùng nghĩa là có hai bên cùng giữ nó (một bên có thể là kẻ
/// trộm), nên server thu hồi **cả họ** và trả 401: kể cả token mới nhất của
/// họ đó cũng không dùng được nữa, và người dùng phải đăng nhập lại.
///
/// Access token **không** bị thu hồi khi họ bị thu hồi hay khi đăng xuất: JWT
/// vẫn hợp lệ tới `exp`, đây là lý do thời hạn của nó phải ngắn.
///
/// Mặc định module không bắt buộc đăng nhập ở route nào. Để `/trips` đòi
/// `Authorization: Bearer`, truyền thêm [requireBearer] vào `middleware`:
///
/// ```dart
/// final auth = AuthModule();
/// final handler = createFakeApiHandler(
///   modules: [auth],
///   middleware: [auth.requireBearer],
/// );
/// ```
final class AuthModule implements FakeApiModule {
  AuthModule({
    this.otpCode = fakeOtpCode,
    this.otpTtl = const Duration(minutes: 5),
    this.maxOtpAttempts = 3,
    this.accessTokenTtl = const Duration(minutes: 15),
    this.refreshTokenTtl = const Duration(days: 30),
    this.protectedPrefixes = const {'/trips'},
    Random? random,
  }) : assert(maxOtpAttempts >= 1),
       _random = random ?? Random.secure();

  /// Mã OTP mà mọi yêu cầu đều chấp nhận (chỉ fake).
  final String otpCode;

  /// OTP hết hạn sau khoảng này kể từ lúc yêu cầu.
  final Duration otpTtl;

  /// Số lần nhập sai tối đa; lần sai thứ [maxOtpAttempts] trả 429 và huỷ yêu cầu.
  final int maxOtpAttempts;

  /// Thời hạn access token. Số liệu minh hoạ.
  final Duration accessTokenTtl;

  /// Thời hạn refresh token. Số liệu minh hoạ.
  final Duration refreshTokenTtl;

  /// Các tiền tố đường dẫn mà [requireBearer] bảo vệ (khớp cả đường dẫn con).
  final Set<String> protectedPrefixes;

  final Random _random;
  final Map<String, _Challenge> _challenges = {};
  final Map<String, String> _userIds = {};
  final Map<String, _AccessRecord> _access = {};
  final Map<String, _RefreshRecord> _refresh = {};
  final List<_Family> _families = [];
  FakeApiContext? _context;
  int _challengeSeq = 0;

  /// Số họ token chưa bị thu hồi (để test xem trạng thái phía server).
  int get activeFamilies => _families.where((f) => !f.revoked).length;

  /// Số họ token đã bị thu hồi, do đăng xuất hoặc do phát hiện dùng lại.
  int get revokedFamilies => _families.where((f) => f.revoked).length;

  FakeApiContext get _ctx =>
      _context ?? (throw StateError('AuthModule chưa được mount'));

  @override
  void mount(Router router, FakeApiContext context) {
    _context = context;
    router.post('/auth/otp/request', _requestOtp);
    router.post('/auth/otp/verify', _verifyOtp);
    router.post('/auth/refresh', _refreshTokens);
    router.post('/auth/logout', _logout);
  }

  /// Middleware đòi `Authorization: Bearer <access token còn hạn>` ở các
  /// đường dẫn trong [protectedPrefixes]; thiếu, sai hoặc hết hạn thì 401.
  ///
  /// Đường dẫn khác (`/health`, `/quotes`, `/auth/...`) đi qua không bị kiểm.
  Middleware get requireBearer =>
      (Handler inner) => (Request request) {
        if (!_isProtected('/${request.url.path}')) {
          return inner(request);
        }
        final header = request.headers['authorization'] ?? '';
        final token = _bearerPattern.firstMatch(header)?.group(1);
        final record = token == null ? null : _access[token];
        if (record == null || !_ctx.clock().isBefore(record.expiresAt)) {
          return problemResponse(request, 401, 'Unauthorized').change(
            headers: const {'www-authenticate': 'Bearer error="invalid_token"'},
          );
        }
        return inner(request);
      };

  /// Đổi access token thành người dùng, hoặc `null` khi token lạ hoặc hết hạn.
  ///
  /// Realtime Gateway giả (`RealtimeModule`) dùng hàm này ở bước bắt tay. Giống
  /// [requireBearer], nó tra nguyên chuỗi token trong bảng đã phát chứ không
  /// đọc payload.
  ({String userId, String role, DateTime expiresAt})? verifyAccessToken(
    String token,
  ) {
    final record = _access[token];
    if (record == null || !_ctx.clock().isBefore(record.expiresAt)) {
      return null;
    }
    return (
      userId: record.userId,
      role: record.role,
      expiresAt: record.expiresAt,
    );
  }

  bool _isProtected(String path) => protectedPrefixes.any(
    (prefix) => path == prefix || path.startsWith('$prefix/'),
  );

  Future<Response> _requestOtp(Request request) async {
    final body = await readJsonObject(request);
    final errors = <String, List<String>>{};
    final phone = body['phone'];
    if (phone is! String || !_phonePattern.hasMatch(phone)) {
      errors['phone'] = [
        'Expected an international number such as +84000000001',
      ];
    }
    final app = body['app'];
    if (app != 'customer' && app != 'driver') {
      errors['app'] = ['Expected one of: customer, driver'];
    }
    if (errors.isNotEmpty) {
      throw ProblemException(422, 'Unprocessable request', errors: errors);
    }
    final requestId = 'otp-${(++_challengeSeq).toString().padLeft(6, '0')}';
    _challenges[requestId] = _Challenge(
      phone! as String,
      app! as String,
      _ctx.clock().add(otpTtl),
    );
    return jsonResponse(200, <String, Object?>{
      'requestId': requestId,
      'expiresInSeconds': otpTtl.inSeconds,
    });
  }

  Future<Response> _verifyOtp(Request request) async {
    final body = await readJsonObject(request);
    final requestId = body['requestId'];
    final code = body['code'];
    if (requestId is! String || code is! String) {
      throw const ProblemException(
        422,
        'Unprocessable request',
        errors: {
          'requestId': ['Required string'],
          'code': ['Required string'],
        },
      );
    }
    final challenge = _challenges[requestId];
    if (challenge == null || !_ctx.clock().isBefore(challenge.expiresAt)) {
      _challenges.remove(requestId);
      throw _invalidCode;
    }
    if (code != otpCode) {
      challenge.wrongAttempts++;
      if (challenge.wrongAttempts >= maxOtpAttempts) {
        _challenges.remove(requestId);
        throw const ProblemException(429, 'Too many attempts');
      }
      throw _invalidCode;
    }
    _challenges.remove(requestId);

    final userId = _userIds.putIfAbsent(
      '${challenge.role}:${challenge.phone}',
      () => 'U-${1000 + _userIds.length + 1}',
    );
    final family = _Family(userId, challenge.role);
    _families.add(family);
    return jsonResponse(200, _issue(family));
  }

  static const _invalidCode = ProblemException(
    422,
    'Unprocessable request',
    errors: {
      'code': ['Invalid or expired code'],
    },
  );

  Future<Response> _refreshTokens(Request request) async {
    final token = await _readRefreshToken(request);
    final record = _refresh[token];
    if (record == null) {
      throw const ProblemException(401, 'Invalid refresh token');
    }
    final family = record.family;
    if (family.revoked) {
      throw const ProblemException(401, 'Refresh token revoked');
    }
    if (record.used) {
      // Hai bên cùng giữ một refresh token đã xoay: thu hồi cả họ.
      family.revoked = true;
      throw const ProblemException(401, 'Refresh token reuse detected');
    }
    if (!_ctx.clock().isBefore(record.expiresAt)) {
      throw const ProblemException(401, 'Refresh token expired');
    }
    record.used = true;
    return jsonResponse(200, _issue(family));
  }

  Future<Response> _logout(Request request) async {
    final token = await _readRefreshToken(request);
    // Token lạ cũng trả 204: đừng cho người ngoài dò xem token nào còn sống.
    _refresh[token]?.family.revoked = true;
    return Response(204);
  }

  Future<String> _readRefreshToken(Request request) async {
    final token = (await readJsonObject(request))['refreshToken'];
    if (token is! String || token.isEmpty) {
      throw const ProblemException(
        422,
        'Unprocessable request',
        errors: {
          'refreshToken': ['Required string'],
        },
      );
    }
    return token;
  }

  Map<String, Object?> _issue(_Family family) {
    final now = _ctx.clock();
    final expiresAt = now.add(accessTokenTtl);
    final access = _jwt(family, now, expiresAt);
    final refresh = 'rt_${_hex(24)}';
    _access[access] = _AccessRecord(family.userId, family.role, expiresAt);
    _refresh[refresh] = _RefreshRecord(family, now.add(refreshTokenTtl));
    return <String, Object?>{
      'accessToken': access,
      'refreshToken': refresh,
      'tokenType': 'Bearer',
      'expiresIn': accessTokenTtl.inSeconds,
      'user': <String, Object?>{'id': family.userId, 'role': family.role},
    };
  }

  /// JWT không ký (`alg: none`, chữ ký để trống), xem mô tả lớp.
  String _jwt(_Family family, DateTime now, DateTime expiresAt) {
    String part(Map<String, Object?> json) =>
        base64Url.encode(utf8.encode(jsonEncode(json))).replaceAll('=', '');
    return '${part(const {'alg': 'none', 'typ': 'JWT'})}.'
        '${part({'iss': 'fake.toinoi.example', 'sub': family.userId, 'role': family.role, 'iat': now.millisecondsSinceEpoch ~/ 1000, 'exp': expiresAt.millisecondsSinceEpoch ~/ 1000, 'jti': _hex(8)})}.';
  }

  String _hex(int bytes) => List.generate(
    bytes,
    (_) => _random.nextInt(256).toRadixString(16).padLeft(2, '0'),
  ).join();
}
