import 'package:dio/dio.dart';

/// Log request và response mà không làm lộ dữ liệu nhạy cảm.
///
/// Nguyên tắc: chỉ ghi thứ đã biết là an toàn.
/// - Header đi theo **danh sách cho phép**: header nằm ngoài [visibleHeaders]
///   chỉ lộ tên, giá trị thành `[redacted]`. Header mới ai đó thêm sau này
///   (`Authorization`, `Cookie`, khoá API) tự động bị che.
/// - Query chỉ ghi tên khoá, không ghi giá trị.
/// - Không bao giờ ghi body của request lẫn response.
/// - Lỗi chỉ ghi loại và mã trạng thái. `DioException.toString()` ghi
///   `message` và `error` do tầng dưới đặt, ta không kiểm soát nội dung đó
///   nên không đưa vào log.
final class SafeLogInterceptor extends Interceptor {
  SafeLogInterceptor(this.sink, {Set<String> visibleHeaders = defaultHeaders})
    : visibleHeaders = {for (final name in visibleHeaders) name.toLowerCase()};

  /// Header mặc định được phép ghi giá trị.
  static const defaultHeaders = {'accept', 'content-type', 'content-length'};

  /// Nơi nhận từng dòng log (ví dụ `print`, một `Logger`, hoặc `list.add`).
  final void Function(String line) sink;

  /// Tên header (chữ thường) được ghi giá trị.
  final Set<String> visibleHeaders;

  static const _stopwatchKey = 'toi_noi_api/stopwatch';

  @override
  void onRequest(RequestOptions options, RequestInterceptorHandler handler) {
    options.extra[_stopwatchKey] = Stopwatch()..start();
    sink(
      '--> ${options.method} ${_target(options)} '
      '${_headers(options.headers.entries.map((e) => MapEntry(e.key, '${e.value}')))}',
    );
    handler.next(options);
  }

  @override
  void onResponse(
    Response<dynamic> response,
    ResponseInterceptorHandler handler,
  ) {
    final options = response.requestOptions;
    sink(
      '<-- ${response.statusCode} ${options.method} ${_target(options)} '
      '(${_elapsed(options)}) ${_headers(response.headers.map.entries.map((e) => MapEntry(e.key, e.value.join(', '))))}',
    );
    handler.next(response);
  }

  @override
  void onError(DioException err, ErrorInterceptorHandler handler) {
    final options = err.requestOptions;
    final status = err.response?.statusCode;
    sink(
      '<-- ERROR ${err.type.name}${status == null ? '' : ' $status'} '
      '${options.method} ${_target(options)} (${_elapsed(options)})',
    );
    handler.next(err);
  }

  String _target(RequestOptions options) {
    final uri = options.uri;
    final keys = uri.queryParametersAll.keys;
    return keys.isEmpty
        ? uri.path
        : '${uri.path}?${keys.map((key) => '$key=[redacted]').join('&')}';
  }

  String _headers(Iterable<MapEntry<String, String>> entries) {
    final parts = [
      for (final entry in entries)
        '${entry.key.toLowerCase()}: '
            '${visibleHeaders.contains(entry.key.toLowerCase()) ? entry.value : '[redacted]'}',
    ];
    return '{${parts.join(', ')}}';
  }

  String _elapsed(RequestOptions options) {
    final stopwatch = options.extra[_stopwatchKey];
    return stopwatch is Stopwatch
        ? '${stopwatch.elapsedMilliseconds} ms'
        : '? ms';
  }
}
