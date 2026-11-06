import 'model.dart';
import 'trip_queries.dart';

extension TripSummaryLong on Iterable<Trip> {
  String get summary => '$length chuyến, tổng $totalFareVnd VND';
}
