import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'booking_controller.dart';
import 'booking_screen.dart';
import 'trip_screen.dart';

class ToiNoiApp extends ConsumerWidget {
  const ToiNoiApp({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final state = ref.watch(bookingProvider);
    return MaterialApp(
      title: 'Tới Nơi',
      debugShowCheckedModeBanner: false,
      theme: ThemeData(colorSchemeSeed: Colors.teal),
      home: switch (state) {
        BookingPlaced() => TripScreen(placed: state),
        _ => const BookingScreen(),
      },
    );
  }
}
