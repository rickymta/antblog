import 'package:dio/dio.dart';
import 'package:toi_noi_api/toi_noi_api.dart';

import 'session_manager.dart';
import 'token_store.dart';

/// Gắn `Authorization: Bearer` vào mọi request, và khi server trả 401 thì
/// refresh **một lần** rồi phát lại request.
///
/// Đây là `Interceptor` thường, không phải `QueuedInterceptor`. Hàng đợi của
/// `QueuedInterceptor` xử lý lỗi từng cái một: trong lúc `onError` đầu tiên
/// chờ refresh, mọi lỗi khác xếp hàng sau nó, và nếu việc chờ đó lại gửi một
/// request đi qua cùng `Dio` rồi nhận lỗi, lỗi mới xếp sau chính task đang chờ
/// nó: treo vĩnh viễn. Ở đây không có hàng đợi; "chỉ một lần refresh" do
/// [SessionManager] đảm bảo bằng một `Future` dùng chung, và lời gọi refresh
/// nằm trên `Dio` riêng của `AuthApi`.
///
/// Thứ tự gắn vào client: log, `AuthInterceptor`, rồi `RetryInterceptor`.
/// `onError` của dio chạy theo đúng thứ tự trong danh sách (không đảo ngược),
/// nên log thấy 401 trước khi ta xử lý nó. Cả lần phát lại sau refresh lẫn lần
/// thử lại của `RetryInterceptor` đều đi qua `dio.fetch`, tức qua cả danh sách
/// một lần nữa, và `onRequest` ở đây **luôn ghi đè** header bằng token hiện
/// tại: lần thử lại sau khi token đã đổi không mang token cũ đi.
final class AuthInterceptor extends Interceptor {
  AuthInterceptor({
    required this.dio,
    required this.session,
    this.publicPaths = const {},
  });

  /// `Dio` mà interceptor này được gắn vào, dùng để phát lại request.
  final Dio dio;

  final SessionManager session;

  /// Đường dẫn (như truyền cho dio, không gồm `baseUrl`) không cần đăng nhập:
  /// không gắn token và không refresh khi lỗi, ví dụ `/quotes`.
  final Set<String> publicPaths;

  static const _replayedKey = 'toi_noi_24/auth_replayed';
  static const _epochKey = 'toi_noi_24/session_epoch';

  bool _isPublic(RequestOptions options) => publicPaths.contains(options.path);

  @override
  Future<void> onRequest(
    RequestOptions options,
    RequestInterceptorHandler handler,
  ) async {
    if (!_isPublic(options)) {
      // Đang có refresh chạy thì chờ nó, để không gửi đi một token sắp chết.
      await session.waitForRefresh();
      final tokens = await session.read();
      // Ghi phiên mà request này thuộc về (không await giữa hai dòng).
      options.extra[_epochKey] = session.epoch;
      if (tokens == null) {
        options.headers.remove('authorization');
      } else {
        options.headers['authorization'] = 'Bearer ${tokens.accessToken}';
      }
    }
    handler.next(options);
  }

  @override
  Future<void> onError(
    DioException err,
    ErrorInterceptorHandler handler,
  ) async {
    final options = err.requestOptions;
    final rejected = _bearerOf(options);
    if (err.response?.statusCode != 401 ||
        _isPublic(options) ||
        options.extra[_replayedKey] == true ||
        options.extra[_epochKey] != session.epoch ||
        rejected == null) {
      // Không phải 401, không cần token, đã phát lại một lần mà vẫn 401, đã
      // đổi phiên từ lúc gửi (đăng xuất hoặc đăng nhập lại: đừng phát lại một
      // request của người khác bằng token của người này), hoặc request chưa
      // từng mang token: không có gì để refresh.
      handler.next(err);
      return;
    }

    final AuthTokens fresh;
    try {
      fresh = await session.refreshAfterUnauthorized(rejected);
    } on SessionExpired {
      // Phiên đã kết thúc và state đăng nhập đã được báo qua `signedOut`;
      // request này nhận lại đúng 401 của nó.
      handler.next(err);
      return;
    } on ApiFailure catch (failure) {
      // Refresh không tới được server (mạng, timeout, 5xx): phiên còn nguyên,
      // request nhận lỗi thật của lần refresh để giao diện cho thử lại.
      handler.next(
        DioException(
          requestOptions: options,
          type: DioExceptionType.unknown,
          error: failure,
        ),
      );
      return;
    }

    final cancelled = options.cancelToken?.cancelError;
    if (cancelled != null) {
      // Người gọi đã huỷ trong lúc chờ refresh: đừng phát lại.
      handler.next(cancelled);
      return;
    }
    try {
      final response = await dio.fetch<dynamic>(
        options.copyWith(
          headers: {
            ...options.headers,
            'authorization': 'Bearer ${fresh.accessToken}',
          },
          extra: {...options.extra, _replayedKey: true},
        ),
      );
      handler.resolve(response);
    } on DioException catch (e) {
      handler.next(e);
    }
  }

  String? _bearerOf(RequestOptions options) {
    final header = options.headers['authorization'];
    if (header is String && header.startsWith('Bearer ')) {
      return header.substring('Bearer '.length);
    }
    return null;
  }
}
