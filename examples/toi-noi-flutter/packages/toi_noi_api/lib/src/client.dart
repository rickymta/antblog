import 'dart:convert';

import 'package:dio/dio.dart';

import 'dto.dart';
import 'errors.dart';

final _keyPattern = RegExp(r'^[A-Za-z0-9_-]{1,128}$');

/// Client HTTP của Tới Nơi dựng trên `dio`.
///
/// Trả DTO, không trả domain: việc đổi sang `Trip` và `Money` là của lớp gọi
/// (xem `TripDto.toDomain`). Lỗi HTTP thành [ApiException], payload sai hợp
/// đồng (kể cả body không phải JSON) thành `DtoFormatException`; lỗi tầng mạng
/// (không tới được server, hết thời gian chờ) vẫn là `DioException` nguyên bản;
/// bọc lời gọi bằng `guardApi` để có `ApiFailure`, và gắn `RetryInterceptor`
/// vào [dio] để thử lại có kiểm soát (bài 20).
///
/// Mọi request dùng `ResponseType.plain`: client tự giải JSON để một body hỏng
/// ở 200 ra `DtoFormatException` thay vì một `DioException` của bộ giải mã, và
/// một 500 có body không phải JSON vẫn giữ được mã trạng thái.
final class ToiNoiApiClient {
  /// [baseUrl] có thể kèm tiền tố đường dẫn của gateway, ví dụ
  /// `https://api.example.test/v1` (không kèm query hay fragment). [adapter]
  /// thay phần gửi byte thật (test truyền `InProcessAdapter` từ
  /// `package:toi_noi_fake_api/testing.dart`).
  ToiNoiApiClient({
    required Uri baseUrl,
    HttpClientAdapter? adapter,
    Iterable<Interceptor> interceptors = const [],
    Duration connectTimeout = const Duration(seconds: 10),
    Duration receiveTimeout = const Duration(seconds: 15),
    Duration? sendTimeout,
  }) : _dio = Dio(
         BaseOptions(
           baseUrl: _normalize(baseUrl),
           connectTimeout: connectTimeout,
           receiveTimeout: receiveTimeout,
           sendTimeout: sendTimeout,
           headers: const {'accept': 'application/json'},
         ),
       ) {
    if (adapter != null) {
      _dio.httpClientAdapter = adapter;
    }
    _dio.interceptors.addAll(interceptors);
  }

  final Dio _dio;

  /// Địa chỉ gốc đã chuẩn hoá (không có `/` ở cuối).
  String get baseUrl => _dio.options.baseUrl;

  /// `dio` bên dưới, cho test hoặc bài sau gắn thêm interceptor.
  Dio get dio => _dio;

  static String _normalize(Uri baseUrl) {
    if (!baseUrl.hasScheme ||
        baseUrl.host.isEmpty ||
        (baseUrl.scheme != 'http' && baseUrl.scheme != 'https')) {
      throw ArgumentError.value(
        baseUrl,
        'baseUrl',
        'phải là địa chỉ http(s) tuyệt đối',
      );
    }
    if (baseUrl.hasQuery || baseUrl.hasFragment) {
      throw ArgumentError.value(
        baseUrl,
        'baseUrl',
        'không được kèm query hay fragment (dio nối path vào cuối chuỗi)',
      );
    }
    final text = baseUrl.toString();
    return text.endsWith('/') ? text.substring(0, text.length - 1) : text;
  }

  /// `POST /quotes`. [cancelToken] cho phép huỷ khi người dùng đổi ý; request
  /// bị huỷ ném `DioException` kiểu `cancel` (xem `toApiFailure`).
  Future<QuoteDto> quote(
    QuoteRequestDto request, {
    CancelToken? cancelToken,
  }) async => _send(
    'POST',
    '/quotes',
    cancelToken: cancelToken,
    body: request.toJson(),
    parse: QuoteDto.fromJson,
    dto: 'QuoteDto',
  );

  /// `POST /trips`. [idempotencyKey] đi trong header `Idempotency-Key`: gửi lại
  /// cùng khoá và cùng nội dung thì server trả đúng chuyến đã tạo thay vì tạo
  /// thêm. Sinh khoá một lần cho mỗi ý định đặt chuyến, không sinh lại khi gửi lại.
  Future<TripDto> createTrip(
    QuoteRequestDto request, {
    required String idempotencyKey,
    CancelToken? cancelToken,
  }) async {
    if (!_keyPattern.hasMatch(idempotencyKey)) {
      throw ArgumentError.value(
        idempotencyKey,
        'idempotencyKey',
        'dùng 1..128 chữ cái ASCII, chữ số, gạch nối hoặc gạch dưới',
      );
    }
    return _send(
      'POST',
      '/trips',
      body: request.toJson(),
      headers: {'idempotency-key': idempotencyKey},
      cancelToken: cancelToken,
      parse: TripDto.fromJson,
      dto: 'TripDto',
    );
  }

  /// `GET /trips/{id}`.
  Future<TripDto> getTrip(String id, {CancelToken? cancelToken}) async => _send(
    'GET',
    _tripPath(id),
    cancelToken: cancelToken,
    parse: TripDto.fromJson,
    dto: 'TripDto',
  );

  /// `POST /trips/{id}/accept`, việc của app tài xế.
  ///
  /// Client không biết tài xế là ai: service thật lấy danh tính từ claim `sub`
  /// của JWT (bài 24), còn fake API đọc header `x-fake-driver-id`. Với fake,
  /// gắn header đó bằng một interceptor khi dựng client của tài xế.
  Future<TripDto> acceptTrip(
    String id, {
    required int expectedVersion,
    CancelToken? cancelToken,
  }) async => _send(
    'POST',
    '${_tripPath(id)}/accept',
    cancelToken: cancelToken,
    body: <String, Object?>{'expectedVersion': expectedVersion},
    parse: TripDto.fromJson,
    dto: 'TripDto',
  );

  /// Giải phóng kết nối. Gọi khi client không còn dùng.
  void close({bool force = false}) => _dio.close(force: force);

  /// Mã chuyến là dữ liệu từ ngoài. `Uri.encodeComponent` mã hoá `/`, `?`, `#`
  /// nhưng để nguyên `.` và `..`; dio chuẩn hoá đường dẫn nên `..` sẽ leo ra
  /// khỏi `/trips`, và mã rỗng gửi tới `/trips/`. Ta từ chối các mã đó (và mã
  /// chứa `/`) thay vì đoán.
  String _tripPath(String id) {
    if (id.isEmpty || id == '.' || id == '..' || id.contains('/')) {
      throw ArgumentError.value(
        id,
        'id',
        'mã chuyến không được rỗng, là "." hay "..", hoặc chứa "/"',
      );
    }
    return '/trips/${Uri.encodeComponent(id)}';
  }

  Future<T> _send<T>(
    String method,
    String path, {
    Object? body,
    Map<String, String>? headers,
    CancelToken? cancelToken,
    required T Function(Object? json) parse,
    required String dto,
  }) async {
    final Response<String> response;
    try {
      response = await _dio.request<String>(
        path,
        data: body,
        cancelToken: cancelToken,
        options: Options(
          method: method,
          headers: headers,
          responseType: ResponseType.plain,
        ),
      );
    } on DioException catch (e) {
      final failed = e.response;
      if (e.type == DioExceptionType.badResponse && failed != null) {
        throw ApiException.fromResponse(failed.statusCode ?? 0, failed.data);
      }
      rethrow;
    }
    final Object? json;
    try {
      json = jsonDecode(response.data ?? '');
    } on FormatException {
      throw DtoFormatException(dto, r'$', 'body không phải JSON hợp lệ');
    }
    return parse(json);
  }
}
