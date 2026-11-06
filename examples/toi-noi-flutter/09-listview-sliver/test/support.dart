import 'dart:async';

import 'package:flutter/material.dart';
import 'package:toi_noi_09/trip_history_source.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

/// Lấy [total] chuyến của bản giả, không phụ thuộc thời gian thật.
Future<List<Trip>> loadTrips(int total) async {
  final source = FakeTripHistorySource(total: total, latency: Duration.zero);
  final page = await source.fetchPage(offset: 0, limit: total);
  return page.trips;
}

/// Nguồn do test điều khiển: mỗi lần `fetchPage` được gọi, test thấy ngay
/// (`offsets`) và quyết định lúc nào trả lời hoặc báo lỗi.
class ManualSource implements TripHistorySource {
  ManualSource(this.all);

  /// Toàn bộ dữ liệu, mới nhất trước.
  final List<Trip> all;

  /// Vị trí `offset` của mọi lần gọi, theo thứ tự gọi.
  final offsets = <int>[];

  final _calls = <_Call>[];

  /// Số lần gọi chưa được trả lời.
  int get pending => _calls.where((c) => !c.completer.isCompleted).length;

  @override
  Future<TripPage> fetchPage({required int offset, required int limit}) {
    offsets.add(offset);
    final call = _Call(offset, limit);
    _calls.add(call);
    return call.completer.future;
  }

  /// Trả lời lần gọi thứ [callIndex] (tính từ 0) bằng đúng lát cắt của [all].
  void respond(int callIndex) {
    final call = _calls[callIndex];
    final end = (call.offset + call.limit).clamp(0, all.length);
    final start = call.offset.clamp(0, end);
    call.completer.complete(
      TripPage(trips: all.sublist(start, end), hasMore: end < all.length),
    );
  }

  /// Trả lời lần gọi thứ [callIndex] bằng [page] tuỳ ý.
  void respondWith(int callIndex, TripPage page) =>
      _calls[callIndex].completer.complete(page);

  /// Báo lỗi cho lần gọi thứ [callIndex].
  void fail(int callIndex) =>
      _calls[callIndex].completer.completeError(Exception('mất mạng'));
}

class _Call {
  _Call(this.offset, this.limit);

  final int offset;
  final int limit;
  final completer = Completer<TripPage>();
}

/// Bọc [child] trong MaterialApp + Scaffold để widget cần Material chạy được.
Widget app(Widget child) => MaterialApp(home: Scaffold(body: child));
