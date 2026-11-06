import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:toi_noi_core/toi_noi_core.dart';
import 'package:toi_noi_ui/toi_noi_ui.dart';

import 'providers.dart';
import 'settings_screen.dart';

/// Quãng đường của các chuyến mẫu; cước do `calculateFare` tính.
const sampleDistancesKm = <double>[5, 2.001, 12.5, 100];

/// Lịch sử chuyến đọc từ SQLite. Mọi thay đổi vào bảng tự hiện lên đây.
class HistoryScreen extends ConsumerWidget {
  const HistoryScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final trips = ref.watch(recentTripsProvider);
    return Scaffold(
      appBar: AppBar(
        title: const Text('Lịch sử chuyến'),
        actions: [
          IconButton(
            tooltip: 'Cài đặt',
            icon: const Icon(Icons.settings),
            onPressed: () => Navigator.of(context).push(
              MaterialPageRoute<void>(builder: (_) => const SettingsScreen()),
            ),
          ),
        ],
      ),
      floatingActionButton: FloatingActionButton.extended(
        onPressed: () => _addSample(ref, trips.value?.length ?? 0),
        icon: const Icon(Icons.add),
        label: const Text('Thêm chuyến mẫu'),
      ),
      body: switch (trips) {
        // Lỗi sau một lần có dữ liệu vẫn giữ `.value` cũ; đừng dùng `.value`
        // để quyết định hiện danh sách, hãy khớp theo AsyncError/AsyncData.
        AsyncError() => _ErrorView(
          onClear: () => ref.read(tripHistoryRepositoryProvider).clear(),
        ),
        AsyncData(:final value) when value.isEmpty => const Center(
          child: Text('Chưa có chuyến nào'),
        ),
        AsyncData(:final value) => ListView.separated(
          itemCount: value.length,
          separatorBuilder: (_, _) => const Divider(height: 1),
          itemBuilder: (_, index) => _TripTile(trip: value[index]),
        ),
        _ => const Center(child: CircularProgressIndicator()),
      },
    );
  }

  Future<void> _addSample(WidgetRef ref, int count) {
    final now = ref.read(clockProvider)();
    final km = sampleDistancesKm[count % sampleDistancesKm.length];
    final trip = Trip.request(
      id: 'T${now.millisecondsSinceEpoch}',
      serviceKind: ServiceKind.values[count % ServiceKind.values.length],
      distanceKm: km,
      requestedAt: now,
    );
    return ref.read(tripHistoryRepositoryProvider).saveAll([trip]);
  }
}

class _TripTile extends StatelessWidget {
  const _TripTile({required this.trip});

  final Trip trip;

  @override
  Widget build(BuildContext context) => ListTile(
    title: Text('${trip.serviceKind.label} · ${trip.distanceKm} km'),
    // Lưu UTC, hiển thị theo múi giờ của thiết bị.
    subtitle: Text(formatLocal(trip.requestedAt)),
    trailing: Column(
      mainAxisAlignment: MainAxisAlignment.center,
      crossAxisAlignment: CrossAxisAlignment.end,
      children: [
        MoneyText(trip.fare),
        TripStatusChip(status: trip.status),
      ],
    ),
  );
}

class _ErrorView extends StatelessWidget {
  const _ErrorView({required this.onClear});

  final VoidCallback onClear;

  @override
  Widget build(BuildContext context) => Center(
    child: Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        const Text('Không đọc được lịch sử trên máy.'),
        const SizedBox(height: ToiNoiSpacing.sm),
        FilledButton(onPressed: onClear, child: const Text('Xoá bộ nhớ đệm')),
      ],
    ),
  );
}

/// `dd/MM/yyyy HH:mm` theo múi giờ thiết bị.
String formatLocal(DateTime utc) {
  final t = utc.toLocal();
  String two(int n) => n.toString().padLeft(2, '0');
  return '${two(t.day)}/${two(t.month)}/${t.year} ${two(t.hour)}:${two(t.minute)}';
}
