import 'dart:async';

import 'package:toi_noi_38/location_sample.dart';
import 'package:toi_noi_38/location_tracker.dart';
import 'package:toi_noi_38/location_uploader.dart';

final DateTime t0 = DateTime.utc(2026, 11, 6, 8);

/// Mẫu thứ [i] của chuyến [tripId], cách nhau 5 giây kể từ [t0].
LocationSample sample(int i, {String tripId = 'T1'}) => LocationSample(
  tripId: tripId,
  latitude: 10.77 + i * 0.0001,
  longitude: 106.70,
  accuracyMeters: 8,
  capturedAt: t0.add(Duration(seconds: 5 * i)),
);

/// Sink giả: ghi lại từng lô; ném [Exception] khi [failing] bật.
final class FakeSink implements LocationSink {
  bool failing = false;
  final List<List<LocationSample>> batches = [];

  List<LocationSample> get sent => [for (final b in batches) ...b];

  @override
  Future<void> send(List<LocationSample> batch) async {
    if (failing) throw Exception('mất mạng');
    batches.add(List.of(batch));
  }
}

/// Tracker giả: mỗi lần `positions` tạo một StreamController để test đẩy mẫu.
final class FakeTracker implements LocationTracker {
  final List<String> startedTrips = [];
  final Map<String, StreamController<LocationSample>> controllers = {};

  @override
  Stream<LocationSample> positions({required String tripId}) {
    startedTrips.add(tripId);
    return (controllers[tripId] = StreamController<LocationSample>()).stream;
  }

  bool hasListener(String tripId) => controllers[tripId]?.hasListener ?? false;

  void emit(String tripId, LocationSample s) => controllers[tripId]!.add(s);
}
