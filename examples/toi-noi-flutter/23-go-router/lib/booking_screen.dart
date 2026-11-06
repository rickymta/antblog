import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:toi_noi_core/toi_noi_core.dart';
import 'package:toi_noi_ui/toi_noi_ui.dart';

import 'auth.dart';
import 'paths.dart';
import 'trips.dart';

/// Tab "Đặt xe". `_km` là state cục bộ của widget: nó chỉ sống tiếp khi đổi
/// tab nếu shell giữ nguyên widget này.
class BookingScreen extends ConsumerStatefulWidget {
  const BookingScreen({super.key});

  @override
  ConsumerState<BookingScreen> createState() => _BookingScreenState();
}

class _BookingScreenState extends ConsumerState<BookingScreen> {
  double _km = 5;
  bool _placing = false;

  Future<void> _place() async {
    setState(() => _placing = true);
    final trip = await ref.read(tripRepositoryProvider).place(_km);
    if (!mounted) return;
    setState(() => _placing = false);
    ref.invalidate(tripListProvider);
    // push: tab Đặt xe vẫn nằm dưới, bấm Back là về đúng chỗ cũ.
    context.push(AppPaths.trip(trip.id));
  }

  @override
  Widget build(BuildContext context) {
    final auth = ref.watch(authProvider);
    final signedIn = auth.value != null;
    return Scaffold(
      appBar: AppBar(title: const Text('Đặt xe')),
      body: Padding(
        padding: const EdgeInsets.all(ToiNoiSpacing.md),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text('Quãng đường: ${_km.round()} km'),
            Slider(
              key: const Key('km-slider'),
              value: _km,
              min: 1,
              max: 30,
              divisions: 29,
              onChanged: (value) => setState(() => _km = value),
            ),
            MoneyText(calculateFare(_km)),
            const SizedBox(height: ToiNoiSpacing.md),
            FilledButton(
              key: const Key('place-trip'),
              // Chưa biết đã đăng nhập hay chưa (đang đọc kho): chưa cho bấm.
              onPressed: _placing || !auth.hasValue
                  ? null
                  // go (không push): /login thay cả khung tab; xem bài viết.
                  : signedIn
                  ? _place
                  : () => context.go(AppPaths.login),
              child: Text(signedIn ? 'Đặt chuyến' : 'Đăng nhập để đặt chuyến'),
            ),
          ],
        ),
      ),
    );
  }
}
