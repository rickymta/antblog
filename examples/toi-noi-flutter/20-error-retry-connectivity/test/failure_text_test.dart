import 'package:dio/dio.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_20/connectivity.dart';
import 'package:toi_noi_20/failure_text.dart';
import 'package:toi_noi_20/system_settings.dart';
import 'package:toi_noi_api/toi_noi_api.dart';

const _retry = FailureAction.retry;
const _settings = FailureAction.openSettings;

void main() {
  group('hành động theo loại lỗi', () {
    test('mất mạng: Thử lại và Mở cài đặt', () {
      expect(describeFailure(const NetworkFailure()).actions, {
        _retry,
        _settings,
      });
    });

    test('quá hạn và 5xx: chỉ Thử lại', () {
      expect(
        describeFailure(const TimeoutFailure(phase: TimeoutPhase.receive))
            .actions,
        {_retry},
      );
      expect(
        describeFailure(const ServerFailure(statusCode: 503, title: 't'))
            .actions,
        {_retry},
      );
    });

    test('4xx: không có nút, vì gửi lại y nguyên cho kết quả y nguyên', () {
      for (final status in [400, 403, 404, 409, 422]) {
        expect(
          describeFailure(ClientFailure(statusCode: status, title: 't'))
              .actions,
          isEmpty,
          reason: '$status',
        );
      }
    });

    test('chứng chỉ sai: không có nút (đừng khuyến khích thử lại)', () {
      final info = describeFailure(
        const NetworkFailure(reason: NetworkReason.badCertificate),
      );
      expect(info.actions, isEmpty);
      expect(info.message, contains('không an toàn'));
    });

    test('payload sai hợp đồng: Thử lại thủ công, nhắc Wi-Fi công cộng', () {
      final info = describeFailure(
        const ParseFailure(dto: 'QuoteDto', field: r'$', message: 'x'),
      );
      expect(info.actions, {_retry});
      expect(info.message, contains('Wi-Fi'));
    });

    test('lỗi không phải ApiFailure: câu chung, không nút', () {
      final info = describeFailure(StateError('bug'));
      expect(info.message, 'Đã có lỗi không mong muốn.');
      expect(info.actions, isEmpty);
    });
  });

  group('câu chữ', () {
    test('mất mạng: câu khác nhau theo connectivity, nút thì như nhau', () {
      final none = describeFailure(const NetworkFailure(), link: LinkType.none);
      final wifi = describeFailure(const NetworkFailure(), link: LinkType.wifi);
      final unknown = describeFailure(const NetworkFailure());
      expect(none.message, contains('không có kết nối mạng'));
      expect(wifi.message, contains('Không tới được máy chủ'));
      expect(unknown.message, wifi.message);
      expect(none.actions, wifi.actions);
    });

    test('mã hỗ trợ lấy từ traceId của 5xx, 404 và 409', () {
      for (final failure in [
        const ServerFailure(statusCode: 500, title: 't', traceId: 'fake-7'),
        const ClientFailure(statusCode: 404, title: 't', traceId: 'fake-7'),
        const ClientFailure(statusCode: 409, title: 't', traceId: 'fake-7'),
      ]) {
        expect(describeFailure(failure).supportCode, 'fake-7');
      }
    });

    test(
      '422 trên distanceKm nói rõ miền 0 đến 100 km, không có mã hỗ trợ',
      () {
        final info = describeFailure(
          const ClientFailure(
            statusCode: 422,
            title: 't',
            traceId: 'fake-9',
            fieldErrors: {
              'distanceKm': ['Expected 0..100 km'],
            },
          ),
        );
        expect(info.message, contains('0 đến 100 km'));
        expect(info.supportCode, isNull);
      },
    );

    test('không đưa title tiếng Anh của server lên giao diện', () {
      final info = describeFailure(
        const ServerFailure(statusCode: 500, title: 'Unexpected server error'),
      );
      expect(info.message, isNot(contains('Unexpected')));
    });
  });

  group('408, 429 và huỷ request', () {
    test('408 và 429 là tạm thời: có nút Thử lại, 429 giữ mã hỗ trợ', () {
      final timeout = describeFailure(
        const ClientFailure(statusCode: 408, title: 't'),
      );
      final busy = describeFailure(
        const ClientFailure(statusCode: 429, title: 't', traceId: 'fake-5'),
      );
      expect(timeout.actions, {_retry});
      expect(busy.actions, {_retry});
      expect(busy.supportCode, 'fake-5');
      expect(busy.message, contains('quá nhiều yêu cầu'));
    });

    test('isCancellation chỉ đúng với DioException kiểu cancel', () {
      expect(
        isCancellation(
          DioException.requestCancelled(
            requestOptions: RequestOptions(path: '/x'),
            reason: 'đổi ý',
          ),
        ),
        isTrue,
      );
      expect(
        isCancellation(
          DioException.connectionError(
            requestOptions: RequestOptions(path: '/x'),
            reason: 'r',
          ),
        ),
        isFalse,
      );
      expect(isCancellation(const NetworkFailure()), isFalse);
      expect(isCancellation(StateError('x')), isFalse);
    });

    test('nút Thử lại không đồng nghĩa retryable: ParseFailure có nút dù retryable là false', () {
      const parse = ParseFailure(dto: 'D', field: 'f', message: 'm');
      expect(parse.retryable, isFalse);
      expect(describeFailure(parse).actions, {_retry});
    });
  });

  group('nhãn nút mở cài đặt theo nền tảng', () {
    test('Android mở cài đặt mạng, iOS nói thẳng là cài đặt ứng dụng', () {
      expect(
        const AppSettingsOpener(platform: TargetPlatform.android).buttonLabel,
        'Mở cài đặt mạng',
      );
      expect(
        const AppSettingsOpener(platform: TargetPlatform.iOS).buttonLabel,
        'Mở cài đặt ứng dụng',
      );
    });
  });
}
