import 'package:flutter/material.dart';
import 'package:toi_noi_core/toi_noi_core.dart';
import 'package:toi_noi_ui/toi_noi_ui.dart';

import 'find_driver_screen.dart';

/// App demo: màn hình tìm tài xế của Tới Nơi.
class FindDriverApp extends StatefulWidget {
  const FindDriverApp({super.key, this.initialTrip});

  /// Chuyến ban đầu; mặc định là một chuyến 5 km vừa được đặt.
  final Trip? initialTrip;

  @override
  State<FindDriverApp> createState() => _FindDriverAppState();
}

class _FindDriverAppState extends State<FindDriverApp> {
  final ThemeData _light = ToiNoiTheme.light();
  final ThemeData _dark = ToiNoiTheme.dark();

  late final Trip _trip =
      widget.initialTrip ??
      Trip.request(
        id: 'trip-0001',
        serviceKind: ServiceKind.ride,
        distanceKm: 5,
        requestedAt: DateTime.utc(2026, 11, 6, 8),
      );

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Tới Nơi',
      theme: _light,
      darkTheme: _dark,
      home: FindDriverScreen(initialTrip: _trip),
    );
  }
}
