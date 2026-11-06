import 'dart:convert';

import 'package:dio/dio.dart';
import 'package:flutter/foundation.dart';
import 'package:toi_noi_api/toi_noi_api.dart';

import 'token_store.dart';

/// Kết quả của `POST /auth/otp/request`.
@immutable
final class OtpChallenge {
  const OtpChallenge({required this.requestId, required this.expiresIn});

  /// Mã của lần yêu cầu, gửi kèm khi xác minh mã.
  final String requestId;

  /// OTP còn dùng được bao lâu kể từ lúc yêu cầu.
  final Duration expiresIn;

  @override
  bool operator ==(Object other) =>
      other is OtpChallenge &&
      other.requestId == requestId &&
      other.expiresIn == expiresIn;

  @override
  int get hashCode => Object.hash(requestId, expiresIn);
}

/// Bốn lời gọi tới `/auth/*`, trên một `Dio` **riêng**.
///
/// `Dio` này không có `AuthInterceptor` (cũng không có `RetryInterceptor`).
/// Hai lý do:
///
/// - `refresh` mà đi qua `AuthInterceptor` thì khi nó bị 401, interceptor lại
///   đi refresh để cứu chính lời gọi refresh: vòng lặp, hoặc treo nếu dùng
///   `QueuedInterceptor`.
/// - Refresh không lặp lại được an toàn: với rotation, lần gửi đầu có thể đã
///   đổi token ở server dù response bị mất. Gửi lại cùng token cũ bị coi là
///   dùng lại. Nên không có thử lại tự động ở đây.
///
/// Mọi lỗi ra khỏi lớp này là một [ApiFailure] (qua `guardApi`).
final class AuthApi {
  AuthApi(this._dio);

  /// Dựng `Dio` riêng cho `/auth/*`. [interceptors] thường chỉ là log.
  factory AuthApi.create({
    required Uri baseUrl,
    HttpClientAdapter? adapter,
    Iterable<Interceptor> interceptors = const [],
    Duration connectTimeout = const Duration(seconds: 10),
    Duration receiveTimeout = const Duration(seconds: 15),
  }) {
    final dio = Dio(
      BaseOptions(
        baseUrl: baseUrl.toString().replaceFirst(RegExp(r'/$'), ''),
        connectTimeout: connectTimeout,
        receiveTimeout: receiveTimeout,
        headers: const {'accept': 'application/json'},
        responseType: ResponseType.plain,
      ),
    );
    if (adapter != null) {
      dio.httpClientAdapter = adapter;
    }
    dio.interceptors.addAll(interceptors);
    return AuthApi(dio);
  }

  final Dio _dio;

  /// `Dio` bên dưới, để test xem cấu hình.
  Dio get dio => _dio;

  /// `POST /auth/otp/request`. [app] là `customer` hoặc `driver`.
  Future<OtpChallenge> requestOtp({
    required String phone,
    required String app,
  }) => guardApi(() async {
    final json = await _post('/auth/otp/request', {'phone': phone, 'app': app});
    final id = json['requestId'];
    final seconds = json['expiresInSeconds'];
    if (id is! String || id.isEmpty) {
      throw DtoFormatException('OtpChallenge', 'requestId', 'cần chuỗi');
    }
    if (seconds is! int || seconds <= 0) {
      throw DtoFormatException(
        'OtpChallenge',
        'expiresInSeconds',
        'cần số dương',
      );
    }
    return OtpChallenge(
      requestId: id,
      expiresIn: Duration(seconds: seconds),
    );
  });

  /// `POST /auth/otp/verify`. Mã sai là `ClientFailure` 422, sai quá số lần
  /// cho phép là `ClientFailure` 429.
  Future<AuthTokens> verifyOtp({
    required String requestId,
    required String code,
  }) => guardApi(
    () async => _tokens(
      await _post('/auth/otp/verify', {'requestId': requestId, 'code': code}),
    ),
  );

  /// `POST /auth/refresh`: đổi refresh token lấy một cặp mới (rotation).
  /// Server từ chối thì là `ClientFailure` 401.
  Future<AuthTokens> refresh(String refreshToken) => guardApi(
    () async =>
        _tokens(await _post('/auth/refresh', {'refreshToken': refreshToken})),
  );

  /// `POST /auth/logout`: báo server thu hồi họ token của [refreshToken].
  Future<void> logout(String refreshToken) => guardApi(
    () =>
        _dio.post<String>('/auth/logout', data: {'refreshToken': refreshToken}),
  );

  Future<Map<String, Object?>> _post(String path, Object body) async {
    final response = await _dio.post<String>(path, data: body);
    final Object? json;
    try {
      json = jsonDecode(response.data ?? '');
    } on FormatException {
      throw DtoFormatException(path, r'$', 'body không phải JSON hợp lệ');
    }
    if (json is! Map<String, Object?>) {
      throw DtoFormatException(path, r'$', 'cần một JSON object');
    }
    return json;
  }

  AuthTokens _tokens(Map<String, Object?> json) {
    final access = json['accessToken'];
    final refresh = json['refreshToken'];
    if (access is! String || access.isEmpty) {
      throw DtoFormatException('TokenResponse', 'accessToken', 'cần chuỗi');
    }
    if (refresh is! String || refresh.isEmpty) {
      throw DtoFormatException('TokenResponse', 'refreshToken', 'cần chuỗi');
    }
    return AuthTokens(accessToken: access, refreshToken: refresh);
  }
}
