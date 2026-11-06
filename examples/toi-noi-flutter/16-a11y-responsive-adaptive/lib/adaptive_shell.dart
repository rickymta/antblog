import 'package:flutter/material.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'history_screen.dart';
import 'quote_screen.dart';
import 'settings_screen.dart';
import 'trip_detail.dart';
import 'window_class.dart';

/// Ba đích của thanh điều hướng.
enum Destination {
  quote('Báo giá', Icons.request_quote_outlined, Icons.request_quote),
  history('Lịch sử', Icons.history, Icons.history),
  settings('Cài đặt', Icons.settings_outlined, Icons.settings);

  const Destination(this.label, this.icon, this.selectedIcon);

  final String label;
  final IconData icon;
  final IconData selectedIcon;
}

/// Đồng hồ tiêm được để test không phụ thuộc giờ thật.
typedef UtcClock = DateTime Function();

DateTime systemUtcNow() => DateTime.now().toUtc();

/// Khung app: chọn bố cục theo [WindowClass] của cả cửa sổ.
///
/// - compact: `NavigationBar` ở đáy, chạm một chuyến thì đẩy trang chi tiết;
/// - medium: `NavigationRail` bên trái, vẫn đẩy trang chi tiết;
/// - expanded: `NavigationRail` và hai cột (danh sách, chi tiết).
///
/// Danh sách chuyến sống ở đây bằng `setState`; quản lý state đúng nghĩa là
/// việc của Phần 3.
class AdaptiveShell extends StatefulWidget {
  const AdaptiveShell({
    super.key,
    this.initialTrips = const [],
    this.clock = systemUtcNow,
  });

  final List<Trip> initialTrips;
  final UtcClock clock;

  @override
  State<AdaptiveShell> createState() => _AdaptiveShellState();
}

class _AdaptiveShellState extends State<AdaptiveShell> {
  late List<Trip> _trips = [...widget.initialTrips];
  Destination _destination = Destination.quote;
  String? _selectedId;
  int _nextNumber = 1046;

  // GlobalKey cho phép nội dung "chuyển nhà" giữa hai nhánh bố cục (xoay
  // máy, đổi cỡ cửa sổ) mà vẫn giữ State, ví dụ quãng đường đang kéo.
  final _contentKey = GlobalKey();

  void _book(ServiceKind kind, double km) {
    final trip = Trip.request(
      id: 'T-${_nextNumber++}',
      serviceKind: kind,
      distanceKm: km,
      requestedAt: widget.clock(),
    ).apply(const SearchStarted());
    setState(() => _trips = [trip, ..._trips]);
    ScaffoldMessenger.of(context)
        .showSnackBar(SnackBar(content: Text('Đã gửi yêu cầu ${trip.id}')));
  }

  void _replace(Trip updated) => setState(() {
    _trips = [for (final t in _trips) t.isSameTrip(updated) ? updated : t];
  });

  void _select(Trip trip, WindowClass windowClass) {
    if (windowClass == WindowClass.expanded) {
      setState(() => _selectedId = trip.id);
      return;
    }
    Navigator.of(context).push(
      MaterialPageRoute<void>(
        builder: (_) => TripDetailPage(trip: trip, onChanged: _replace),
      ),
    );
  }

  Widget _history(WindowClass windowClass) {
    final list = HistoryScreen(
      trips: _trips,
      selectedId: _selectedId,
      onSelect: (trip) => _select(trip, windowClass),
    );
    if (windowClass != WindowClass.expanded) return list;

    final selected = _trips.where((t) => t.id == _selectedId).firstOrNull;
    return Row(
      children: [
        SizedBox(width: 360, child: list),
        const VerticalDivider(width: 1),
        Expanded(
          child: selected == null
              ? const Center(child: Text('Chọn một chuyến để xem chi tiết'))
              : TripDetailView(
                  key: ValueKey(selected.id),
                  trip: selected,
                  onCancelled: _replace,
                ),
        ),
      ],
    );
  }

  @override
  Widget build(BuildContext context) {
    // Quyết định theo cả cửa sổ nên đọc MediaQuery; quyết định theo chỗ cha
    // cấp cho widget thì dùng LayoutBuilder.
    final windowClass = WindowClass.fromWidth(MediaQuery.sizeOf(context).width);
    final selectedIndex = _destination.index;

    // IndexedStack giữ state của từng đích (quãng đường đang kéo ở màn báo
    // giá không mất khi sang tab khác).
    final content = IndexedStack(
      key: _contentKey,
      index: selectedIndex,
      children: [
        QuoteScreen(onBook: _book),
        _history(windowClass),
        const SettingsScreen(),
      ],
    );

    void onSelected(int index) =>
        setState(() => _destination = Destination.values[index]);

    if (windowClass == WindowClass.compact) {
      return Scaffold(
        appBar: AppBar(title: Text(_destination.label)),
        body: SafeArea(child: content),
        bottomNavigationBar: NavigationBar(
          selectedIndex: selectedIndex,
          onDestinationSelected: onSelected,
          destinations: [
            for (final d in Destination.values)
              NavigationDestination(
                icon: Icon(d.icon),
                selectedIcon: Icon(d.selectedIcon),
                label: d.label,
              ),
          ],
        ),
      );
    }

    return Scaffold(
      appBar: AppBar(title: Text(_destination.label)),
      body: Row(
        children: [
          // NavigationRail tự có SafeArea ở cạnh bắt đầu, nên phần nội dung
          // chỉ cần chừa các cạnh còn lại.
          NavigationRail(
            selectedIndex: selectedIndex,
            onDestinationSelected: onSelected,
            labelType: NavigationRailLabelType.all,
            destinations: [
              for (final d in Destination.values)
                NavigationRailDestination(
                  icon: Icon(d.icon),
                  selectedIcon: Icon(d.selectedIcon),
                  label: Text(d.label),
                ),
            ],
          ),
          const VerticalDivider(width: 1),
          Expanded(child: SafeArea(left: false, child: content)),
        ],
      ),
    );
  }
}
