import 'dart:convert';

import 'package:dio/dio.dart';
// Bản viết tay của QuoteDto/TripDto cùng tên với bản sinh mã: ẩn đi.
import 'package:toi_noi_api/toi_noi_api.dart' hide QuoteDto, TripDto;
import 'package:toi_noi_core/toi_noi_core.dart';

import 'dto/quote_dto.dart';
import 'dto/trip_dto.dart';

/// Cổng tới backend của báo giá và chuyến, trả DTO sinh mã.
///
/// Mọi lỗi nhận ra được ra khỏi cổng này dưới dạng [ApiFailure].
abstract interface class QuoteSource {
  Future<QuoteDto> quote({
    required ServiceKind serviceKind,
    required double distanceKm,
  });

  Future<TripDto> trip(String id);
}

/// Cài đặt bằng `dio` của [QuoteSource]; giải JSON bằng DTO sinh mã.
///
/// Dùng chung `Dio` của `ToiNoiApiClient` (timeout, interceptor, adapter) và
/// lặp lại đúng quy ước của client đó: `ResponseType.plain`, lỗi HTTP thành
/// [ApiException], body hỏng thành [DtoFormatException], rồi `guardApi` đổi
/// tất cả thành [ApiFailure].
final class DioQuoteSource implements QuoteSource {
  DioQuoteSource(this._dio);

  final Dio _dio;

  @override
  Future<QuoteDto> quote({
    required ServiceKind serviceKind,
    required double distanceKm,
  }) => guardApi(
    () => _send(
      'POST',
      '/quotes',
      body: QuoteRequestDto(
        serviceKind: serviceKind,
        distanceKm: distanceKm,
      ).toJson(),
      parse: QuoteDto.fromJson,
      dto: 'QuoteDto',
    ),
  );

  @override
  Future<TripDto> trip(String id) => guardApi(
    () => _send(
      'GET',
      '/trips/${Uri.encodeComponent(id)}',
      parse: TripDto.fromJson,
      dto: 'TripDto',
    ),
  );

  Future<T> _send<T>(
    String method,
    String path, {
    Object? body,
    required T Function(Object? json) parse,
    required String dto,
  }) async {
    final Response<String> response;
    try {
      response = await _dio.request<String>(
        path,
        data: body,
        options: Options(method: method, responseType: ResponseType.plain),
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
