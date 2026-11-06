import 'package:dio/dio.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_19/error_text.dart';
import 'package:toi_noi_api/toi_noi_api.dart';

void main() {
  test('422 trên distanceKm: nói rõ miền 0 đến 100 km', () {
    final text = describeError(
      const ApiException(
        statusCode: 422,
        title: 'Unprocessable request',
        fieldErrors: {
          'distanceKm': ['Expected 0..100 km'],
        },
        traceId: 'fake-000009',
      ),
    );
    expect(text.message, contains('0 đến 100 km'));
    // Lỗi do người dùng nhập: mã hỗ trợ không giúp gì.
    expect(text.supportCode, isNull);
  });

  test('409, 404, 5xx có câu riêng và mang traceId theo', () {
    for (final (status, fragment) in [
      (409, 'làm mới'),
      (404, 'Không tìm thấy'),
      (500, 'Máy chủ đang gặp sự cố'),
      (503, 'Máy chủ đang gặp sự cố'),
    ]) {
      final text = describeError(
        ApiException(statusCode: status, title: 't', traceId: 'fake-1'),
      );
      expect(text.message, contains(fragment), reason: '$status');
      expect(text.supportCode, 'fake-1', reason: '$status');
    }
  });

  test('4xx khác (ví dụ 403) rơi vào câu chung', () {
    final text = describeError(const ApiException(statusCode: 403, title: 'x'));
    expect(text.message, 'Yêu cầu không được chấp nhận.');
  });

  test('không đưa nguyên title tiếng Anh của server lên giao diện', () {
    final text = describeError(
      const ApiException(statusCode: 500, title: 'Unexpected server error'),
    );
    expect(text.message, isNot(contains('Unexpected')));
  });

  test('payload sai hợp đồng, lỗi tầng mạng và lỗi lạ', () {
    expect(
      describeError(DtoFormatException('TripDto', 'id', 'thiếu trường'))
          .message,
      contains('không đọc được'),
    );
    expect(
      describeError(
        DioException(
          requestOptions: RequestOptions(path: '/x'),
          type: DioExceptionType.connectionError,
        ),
      ).message,
      contains('Không kết nối được'),
    );
    expect(describeError(StateError('?')).message, contains('không mong muốn'));
  });
}
