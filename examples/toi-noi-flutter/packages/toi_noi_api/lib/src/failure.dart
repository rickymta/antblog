// Lộ trình Flutter qua Tới Nơi chỉ nhắm iOS và Android, nên import trực tiếp
// dart:io để nhận ra lỗi socket và TLS. Package này không dùng được cho Flutter web.
import 'dart:io' show HttpException, SocketException, TlsException;

import 'package:dio/dio.dart';

import 'errors.dart';

/// Giai đoạn của request bị quá hạn.
enum TimeoutPhase {
  /// Không mở được kết nối tới server kịp lúc.
  connect,

  /// Gửi request đi quá chậm.
  send,

  /// Chờ dữ liệu từ server quá lâu.
  receive,
}

/// Vì sao không tới được server.
enum NetworkReason {
  /// Không có đường tới server: mất mạng, DNS hỏng, server từ chối kết nối,
  /// kết nối đứt giữa chừng. Thử lại có thể có ích.
  unreachable,

  /// Bắt tay TLS thất bại hoặc chứng chỉ của server bị từ chối. Thử lại không
  /// giúp gì, và không nên tự động bỏ qua.
  badCertificate,
}

/// Mọi thất bại khi gọi API, đã phân loại theo điều người dùng làm được.
///
/// Năm loại, đóng kín (`sealed`) để `switch` bắt buộc xử lý đủ:
/// [NetworkFailure], [TimeoutFailure], [ServerFailure], [ClientFailure],
/// [ParseFailure]. Dùng [toApiFailure] hoặc [guardApi] để đổi
/// `DioException`, [ApiException] và [DtoFormatException] sang đây.
///
/// Chỉ mang dữ kiện kỹ thuật, không có câu hiển thị: chữ trên giao diện thuộc
/// về lớp trình bày (và bài về localization).
sealed class ApiFailure implements Exception {
  const ApiFailure({this.cause});

  /// Lỗi gốc, để ghi log. Có thể là `DioException`, [ApiException] hoặc
  /// [DtoFormatException].
  final Object? cause;

  /// Đây là lỗi tạm thời: **máy** có thể tự thử lại mà không lo sai, miễn là
  /// request lặp lại được (điều đó do `RetryInterceptor.isRepeatable` xét, không
  /// phải do lỗi). `ParseFailure` và 4xx thường là `false`.
  ///
  /// Không phải quy tắc cho nút trên giao diện. Nút Thử lại là quyết định riêng
  /// của lớp trình bày, vì người dùng có thể cân nhắc khác máy (ví dụ thử lại
  /// tay một `ParseFailure` sau khi đăng nhập Wi-Fi công cộng).
  bool get retryable;
}

/// Không có response: không tới được server.
final class NetworkFailure extends ApiFailure {
  const NetworkFailure({this.reason = NetworkReason.unreachable, super.cause});

  final NetworkReason reason;

  @override
  bool get retryable => reason == NetworkReason.unreachable;

  @override
  String toString() => 'NetworkFailure(${reason.name})';
}

/// Quá hạn ở một giai đoạn của request.
final class TimeoutFailure extends ApiFailure {
  const TimeoutFailure({required this.phase, this.limit, super.cause});

  final TimeoutPhase phase;

  /// Hạn đã đặt, nếu biết.
  final Duration? limit;

  @override
  bool get retryable => true;

  @override
  String toString() =>
      'TimeoutFailure(${phase.name}${limit == null ? '' : ', ${limit!.inMilliseconds} ms'})';
}

/// Server trả 5xx: lỗi của server, thường tạm thời.
final class ServerFailure extends ApiFailure {
  const ServerFailure({
    required this.statusCode,
    required this.title,
    this.traceId,
    super.cause,
  });

  final int statusCode;

  /// Tiêu đề do server đặt (tiếng Anh, cho lập trình viên); đừng đưa lên giao diện.
  final String title;

  /// Mã để đối chiếu với log phía server.
  final String? traceId;

  @override
  bool get retryable => true;

  @override
  String toString() =>
      'ServerFailure($statusCode${traceId == null ? '' : ', trace $traceId'})';
}

/// Server trả mã không phải 2xx và dưới 500 (thực tế là 4xx): request sai
/// hoặc không được phép, nên gửi lại y nguyên thường cho kết quả y nguyên.
/// Ngoại lệ là 408 (server chờ request quá lâu) và 429 (quá nhiều request):
/// hai mã này nói "thử lại sau", nên [retryable] là `true`.
final class ClientFailure extends ApiFailure {
  const ClientFailure({
    required this.statusCode,
    required this.title,
    this.fieldErrors = const {},
    this.traceId,
    super.cause,
  });

  final int statusCode;
  final String title;

  /// Tên trường -> thông báo, khi server trả lỗi kiểm tra đầu vào.
  final Map<String, List<String>> fieldErrors;
  final String? traceId;

  @override
  bool get retryable => statusCode == 408 || statusCode == 429;

  @override
  String toString() =>
      'ClientFailure($statusCode${traceId == null ? '' : ', trace $traceId'})';
}

/// Server trả 2xx nhưng nội dung sai hợp đồng ([DtoFormatException]).
final class ParseFailure extends ApiFailure {
  const ParseFailure({
    required this.dto,
    required this.field,
    required this.message,
    super.cause,
  });

  /// Tên DTO, ví dụ `TripDto`.
  final String dto;

  /// Tên trường JSON lỗi, hoặc `$` khi cả payload sai.
  final String field;

  /// Thông báo kỹ thuật, đã gồm tên DTO và trường.
  final String message;

  @override
  bool get retryable => false;

  @override
  String toString() => 'ParseFailure($message)';
}

/// Đổi một lỗi thành [ApiFailure].
///
/// - [ApiFailure]: trả nguyên.
/// - [ApiException]: 5xx thành [ServerFailure], còn lại thành [ClientFailure].
/// - [DtoFormatException]: [ParseFailure].
/// - `DioException`: theo `type`. Hết thời gian chờ thành [TimeoutFailure];
///   `badResponse` đọc như [ApiException]; lỗi mạng thành [NetworkFailure]
///   theo [networkReasonOf]: `connectionError` và `HttpException` /
///   `SocketException` bên trong `unknown` (kết nối đứt giữa body, bị reset) là
///   `unreachable`; `badCertificate` và `TlsException` (kể cả
///   `HandshakeException`, thứ mà dio để ở `unknown`) là `badCertificate`.
///   `unknown` còn lại chỉ được đổi khi lỗi gốc của nó đã là loại ở trên (ví dụ
///   một interceptor ném ra).
///
/// Hai trường hợp được ném lại nguyên vẹn thay vì đổi: `DioException` kiểu
/// `cancel` (huỷ là quyết định của người gọi, không phải thất bại) và mọi lỗi
/// không nhận ra (một `StateError`, một `TypeError`...). Che chúng thành "lỗi
/// mạng" sẽ giấu lỗi lập trình.
ApiFailure toApiFailure(Object error, [StackTrace? stackTrace]) {
  switch (error) {
    case ApiFailure():
      return error;
    case ApiException():
      return _fromApiException(error);
    case DtoFormatException():
      return ParseFailure(
        dto: error.dto,
        field: error.field,
        message: error.message,
        cause: error,
      );
    case DioException():
      return _fromDio(error, stackTrace);
    default:
      Error.throwWithStackTrace(error, stackTrace ?? StackTrace.current);
  }
}

ApiFailure _fromApiException(ApiException e) => e.statusCode >= 500
    ? ServerFailure(
        statusCode: e.statusCode,
        title: e.title,
        traceId: e.traceId,
        cause: e,
      )
    : ClientFailure(
        statusCode: e.statusCode,
        title: e.title,
        fieldErrors: e.fieldErrors,
        traceId: e.traceId,
        cause: e,
      );

ApiFailure _fromDio(DioException e, StackTrace? stackTrace) {
  switch (e.type) {
    case DioExceptionType.connectionTimeout:
      return TimeoutFailure(
        phase: TimeoutPhase.connect,
        limit: e.requestOptions.connectTimeout,
        cause: e,
      );
    case DioExceptionType.sendTimeout:
      return TimeoutFailure(
        phase: TimeoutPhase.send,
        limit: e.requestOptions.sendTimeout,
        cause: e,
      );
    case DioExceptionType.receiveTimeout:
    case DioExceptionType.transformTimeout:
      return TimeoutFailure(
        phase: TimeoutPhase.receive,
        limit: e.requestOptions.receiveTimeout,
        cause: e,
      );
    case DioExceptionType.connectionError:
    case DioExceptionType.badCertificate:
      return NetworkFailure(reason: networkReasonOf(e)!, cause: e);
    case DioExceptionType.badResponse:
      final response = e.response;
      return _fromApiException(
        ApiException.fromResponse(response?.statusCode ?? 0, response?.data),
      );
    case DioExceptionType.cancel:
      Error.throwWithStackTrace(e, stackTrace ?? e.stackTrace);
    case DioExceptionType.unknown:
      final reason = networkReasonOf(e);
      if (reason != null) {
        return NetworkFailure(reason: reason, cause: e);
      }
      final inner = e.error;
      if (inner is ApiFailure ||
          inner is ApiException ||
          inner is DtoFormatException) {
        return toApiFailure(inner!, stackTrace);
      }
      Error.throwWithStackTrace(e, stackTrace ?? e.stackTrace);
  }
}

/// Lý do mạng của một `DioException`, hoặc `null` nếu đây không phải lỗi mạng.
///
/// Dùng chung cho [toApiFailure] và `RetryInterceptor` để hai nơi không bao giờ
/// lệch nhau về chuyện "lỗi này có phải mất mạng không".
///
/// Adapter của dio chỉ gán `connectionError` cho một phần lỗi socket. Đã thử với
/// socket thật ở localhost: kết nối đóng trước khi có header là
/// `connectionError`, nhưng kết nối đứt giữa body là `unknown` mang
/// `HttpException`, và bắt tay TLS hỏng là `unknown` mang `HandshakeException`.
/// `badCertificate` chỉ phát sinh khi client đặt `validateCertificate`.
NetworkReason? networkReasonOf(DioException e) {
  switch (e.type) {
    case DioExceptionType.connectionError:
      return NetworkReason.unreachable;
    case DioExceptionType.badCertificate:
      return NetworkReason.badCertificate;
    case DioExceptionType.unknown:
      final inner = e.error;
      if (inner is TlsException) {
        return NetworkReason.badCertificate;
      }
      if (inner is HttpException || inner is SocketException) {
        return NetworkReason.unreachable;
      }
      return null;
    case DioExceptionType.connectionTimeout:
    case DioExceptionType.sendTimeout:
    case DioExceptionType.receiveTimeout:
    case DioExceptionType.transformTimeout:
    case DioExceptionType.badResponse:
    case DioExceptionType.cancel:
      return null;
  }
}

/// Chạy [call] và đổi mọi lỗi nhận ra được thành [ApiFailure].
///
/// ```dart
/// final quote = await guardApi(() => client.quote(request));
/// ```
///
/// Ngăn chặn việc `catch (e)` rải rác: tầng trên chỉ cần `on ApiFailure`
/// hoặc `switch` trên năm loại.
Future<T> guardApi<T>(Future<T> Function() call) async {
  try {
    return await call();
  } catch (error, stackTrace) {
    Error.throwWithStackTrace(toApiFailure(error, stackTrace), stackTrace);
  }
}
