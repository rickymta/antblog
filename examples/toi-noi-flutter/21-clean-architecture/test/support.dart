import 'dart:convert';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_riverpod/misc.dart' show Override;
import 'package:shelf/shelf.dart';
import 'package:toi_noi_21/data/trip_repository_impl.dart';
import 'package:toi_noi_21/domain/domain.dart';
import 'package:toi_noi_21/presentation/providers.dart';
import 'package:toi_noi_api/fault_injection.dart';
import 'package:toi_noi_api/toi_noi_api.dart';
import 'package:toi_noi_core/toi_noi_core.dart';
import 'package:toi_noi_fake_api/testing.dart';
import 'package:toi_noi_fake_api/toi_noi_fake_api.dart';

/// Mốc thời gian cố định để test không phụ thuộc đồng hồ thật.
final fixedNow = DateTime.utc(2026, 11, 6, 3, 30);

const ride5km = QuoteRequest(serviceKind: ServiceKind.ride, distanceKm: 5);
const food3km = QuoteRequest(serviceKind: ServiceKind.food, distanceKm: 3);

/// Chuyến mà repository giả trả về khi test không quan tâm nội dung.
Trip tripFor(QuoteRequest request, {String id = 'T-1'}) => Trip.request(
  id: id,
  serviceKind: request.serviceKind,
  distanceKm: request.distanceKm,
  requestedAt: fixedNow,
);

/// `TripRepository` giả, thuần Dart: không dio, không adapter, không fake API.
///
/// Ghi lại mọi lời gọi; hành vi chỉnh bằng `onQuote`, `onRequest`, `onGet`.
final class FakeTripRepository implements TripRepository {
  Future<Money> Function(QuoteRequest request)? onQuote;
  Future<Trip> Function(QuoteRequest request, String intentId)? onRequest;
  Future<Trip> Function(String id)? onGet;

  final List<QuoteRequest> quoted = [];
  final List<({QuoteRequest request, String intentId})> requested = [];
  final List<String> fetched = [];

  @override
  Future<Money> quote(QuoteRequest request) {
    quoted.add(request);
    final handler = onQuote;
    return handler != null
        ? handler(request)
        : Future.value(calculateFare(request.distanceKm));
  }

  @override
  Future<Trip> requestTrip(QuoteRequest request, {required String intentId}) {
    requested.add((request: request, intentId: intentId));
    final handler = onRequest;
    return handler != null
        ? handler(request, intentId)
        : Future.value(tripFor(request, id: 'T-${requested.length}'));
  }

  @override
  Future<Trip> getTrip(String id) {
    fetched.add(id);
    final handler = onGet;
    return handler != null
        ? handler(id)
        : Future.value(tripFor(ride5km, id: id));
  }
}

/// Một "máy chủ" cho test: fake API chạy trong tiến trình, bọc bởi
/// [FaultyAdapter] để tiêm lỗi tầng mạng, nối vào [TripRepositoryImpl] thật.
///
/// Không có `RetryInterceptor`: test muốn thấy đúng một lần gọi, đúng một lỗi.
final class Harness {
  Harness._(this.store, this.inProcess, this.faulty, this.repository);

  factory Harness({
    FaultConfig faults = const FaultConfig(),
    Iterable<NetworkFault?> script = const [],
    Handler Function(Handler fake)? wrap,
    Duration receiveTimeout = const Duration(milliseconds: 20),
    Duration connectTimeout = const Duration(milliseconds: 20),
  }) {
    final store = FakeApiStore();
    var seq = 0;
    final fake = createFakeApiHandler(
      faults: faults,
      store: store,
      clock: () => fixedNow,
      newId: () => 'T-${++seq}',
    );
    final inProcess = InProcessAdapter(wrap == null ? fake : wrap(fake));
    final faulty = FaultyAdapter(inProcess, script: script);
    final client = ToiNoiApiClient(
      baseUrl: Uri.parse('http://fake.test'),
      adapter: faulty,
      connectTimeout: connectTimeout,
      receiveTimeout: receiveTimeout,
    );
    return Harness._(store, inProcess, faulty, TripRepositoryImpl(client));
  }

  final FakeApiStore store;

  /// Chỉ ghi những request thật sự tới "server".
  final InProcessAdapter inProcess;

  /// Mọi lần gọi tầng mạng, kể cả lần bị tiêm lỗi.
  final FaultyAdapter faulty;

  final TripRepositoryImpl repository;
}

/// Response JSON với [body].
Response okJson(Object body, {int status = 200}) => Response(
  status,
  body: jsonEncode(body),
  headers: const {'content-type': 'application/json; charset=utf-8'},
);

/// Override cho presentation: cắm [repository] vào chỗ cắm, `intentId` dễ đoán.
List<Override> overridesFor(TripRepository repository) {
  var n = 0;
  return [
    tripRepositoryProvider.overrideWithValue(repository),
    intentIdProvider.overrideWithValue(() => 'book-${++n}'),
  ];
}

ProviderContainer containerFor(TripRepository repository) =>
    ProviderContainer.test(
      overrides: overridesFor(repository),
      retry: (_, _) => null,
    );
