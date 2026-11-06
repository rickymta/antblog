import 'package:flutter/material.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'clock.dart';
import 'confirm_trip_screen.dart';
import 'quote.dart';
import 'routes.dart';

/// Màn hình báo giá: mở màn hình xác nhận và nhận lại `Trip?`.
class QuoteScreen extends StatefulWidget {
  const QuoteScreen({super.key, required this.quote, this.clock = systemClock});

  final Quote quote;
  final Clock clock;

  @override
  State<QuoteScreen> createState() => _QuoteScreenState();
}

class _QuoteScreenState extends State<QuoteScreen> {
  Trip? _trip;
  String _status = 'Chưa đặt chuyến';

  Future<void> _book() async {
    final trip = await Navigator.of(context).push<Trip>(
      MaterialPageRoute<Trip>(
        builder: (_) =>
            ConfirmTripScreen(quote: widget.quote, clock: widget.clock),
      ),
    );
    // Await có thể kéo dài; nếu màn hình này đã bị gỡ thì dừng tại đây,
    // vì setState và ScaffoldMessenger.of(context) đều cần State còn sống.
    if (!mounted) return;
    setState(() {
      _trip = trip;
      _status = trip == null
          ? 'Bạn đã quay lại, chưa đặt chuyến'
          : 'Đã đặt chuyến ${trip.id}: ${trip.fare}';
    });
    if (trip != null) {
      ScaffoldMessenger.of(
        context,
      ).showSnackBar(SnackBar(content: Text('Đang tìm tài xế cho ${trip.id}')));
    }
  }

  @override
  Widget build(BuildContext context) {
    final quote = widget.quote;
    return Scaffold(
      appBar: AppBar(
        title: const Text('Báo giá'),
        actions: [
          IconButton(
            tooltip: 'Lịch sử chuyến',
            icon: const Icon(Icons.history),
            onPressed: () => Navigator.of(context).pushNamed(AppRoutes.history),
          ),
        ],
      ),
      body: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              quote.serviceKind.label,
              style: Theme.of(context).textTheme.titleMedium,
            ),
            Text('${quote.pickup} → ${quote.dropoff}'),
            Text('${quote.distanceKm} km'),
            const SizedBox(height: 12),
            Text(
              '${quote.fare}',
              style: Theme.of(context).textTheme.headlineMedium,
            ),
            const SizedBox(height: 24),
            FilledButton(onPressed: _book, child: const Text('Đặt chuyến')),
            const SizedBox(height: 16),
            Text(_status, key: const Key('status')),
            if (_trip != null) Text(_trip!.status.label),
          ],
        ),
      ),
    );
  }
}
