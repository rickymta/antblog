import 'dart:async';

import 'package:toi_noi_26/dto/quote_dto.dart';
import 'package:toi_noi_26/dto/trip_dto.dart';
import 'package:toi_noi_26/quote_source.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'fixtures.dart';

/// [QuoteSource] giả, điều khiển được bằng tay: mỗi lời gọi chờ một
/// [Completer] để test quyết định lúc nào server "trả lời".
final class FakeQuoteSource implements QuoteSource {
  final List<Completer<QuoteDto>> quoteCalls = [];
  final List<String> tripCalls = [];
  final Map<String, Completer<TripDto>> _trips = {};

  @override
  Future<QuoteDto> quote({
    required ServiceKind serviceKind,
    required double distanceKm,
  }) {
    final completer = Completer<QuoteDto>();
    quoteCalls.add(completer);
    return completer.future;
  }

  @override
  Future<TripDto> trip(String id) {
    tripCalls.add(id);
    return (_trips[id] ??= Completer<TripDto>()).future;
  }

  /// Trả lời lời gọi báo giá thứ [index] bằng đúng số tiền [fare].
  void answerQuote(int index, Money fare) => quoteCalls[index].complete(
    QuoteDto.fromJson(changed(validQuoteJson(), {'fare': fare.vnd})),
  );

  void answerTrip(String id, TripDto dto) => _trips[id]!.complete(dto);

  void failTrip(String id, Object error) => _trips[id]!.completeError(error);
}
