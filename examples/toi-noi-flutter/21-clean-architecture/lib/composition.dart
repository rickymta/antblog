import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/misc.dart' show Override;
import 'package:toi_noi_21/data/trip_repository_impl.dart';
import 'package:toi_noi_21/presentation/providers.dart';
import 'package:toi_noi_api/toi_noi_api.dart';

/// Composition root: nơi duy nhất biết cả ba lớp và nối chúng với nhau.
///
/// Địa chỉ fake API. Bài 25 thay chỗ này bằng cấu hình theo flavor.
/// Truyền lúc chạy: `--dart-define=API_BASE_URL=http://10.0.2.2:8787`.
final Uri apiBaseUrl = Uri.parse(
  const String.fromEnvironment(
    'API_BASE_URL',
    defaultValue: 'http://localhost:8787',
  ),
);

/// Ghi đè `tripRepositoryProvider` bằng thực thi gọi API thật.
Override productionTripRepository({Uri? baseUrl}) =>
    tripRepositoryProvider.overrideWith((ref) {
      final client = ToiNoiApiClient(
        baseUrl: baseUrl ?? apiBaseUrl,
        interceptors: [SafeLogInterceptor(debugPrint)],
      );
      client.dio.interceptors.add(RetryInterceptor(dio: client.dio));
      ref.onDispose(client.close);
      return TripRepositoryImpl(client);
    });

/// Tắt retry mặc định của Riverpod: `RetryInterceptor` đã thử lại đúng chỗ
/// (bài 20). Truyền cho `ProviderScope(retry:)`.
Duration? noProviderRetry(int retryCount, Object error) => null;
