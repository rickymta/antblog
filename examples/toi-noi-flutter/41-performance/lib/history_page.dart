import 'package:flutter/material.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'trip_json.dart';

/// Hàm đọc lịch sử từ JSON; tiêm vào để test chọn isolate hay không.
typedef TripsParser = Future<List<Trip>> Function(String json);

/// Trang thử tay: nạp lịch sử lớn trên main isolate hoặc isolate khác, rồi
/// hiện bằng `ListView.builder`. Dùng để quan sát bằng DevTools ở profile mode.
class HistoryPage extends StatefulWidget {
  /// Tạo trang với [tripCount] chuyến.
  const HistoryPage({this.tripCount = 20000, this.parser, super.key});

  /// Số chuyến sinh ra.
  final int tripCount;

  /// Thay cho việc tự chọn parser; `null` thì chọn theo công tắc.
  final TripsParser? parser;

  @override
  State<HistoryPage> createState() => _HistoryPageState();
}

class _HistoryPageState extends State<HistoryPage> {
  bool _useIsolate = true;
  bool _loading = false;
  List<Trip> _trips = const [];

  Future<void> _load() async {
    setState(() => _loading = true);
    final json = generateTripsJson(widget.tripCount);
    final parse =
        widget.parser ??
        (_useIsolate ? parseTripsInIsolate : (String s) async => parseTrips(s));
    final trips = await parse(json);
    if (!mounted) {
      return;
    }
    setState(() {
      _trips = trips;
      _loading = false;
    });
  }

  @override
  Widget build(BuildContext context) => Scaffold(
    appBar: AppBar(title: const Text('Lịch sử chuyến')),
    body: Column(
      children: [
        SwitchListTile(
          title: const Text('Parse trên isolate khác'),
          value: _useIsolate,
          onChanged: (v) => setState(() => _useIsolate = v),
        ),
        FilledButton(
          onPressed: _loading ? null : _load,
          child: Text('Tải ${widget.tripCount} chuyến'),
        ),
        if (_loading) const LinearProgressIndicator(),
        Text('Đã nạp ${_trips.length} chuyến'),
        Expanded(
          child: ListView.builder(
            itemCount: _trips.length,
            itemBuilder: (context, i) => ListTile(
              title: Text(_trips[i].id),
              trailing: Text(_trips[i].fare.toString()),
            ),
          ),
        ),
      ],
    ),
  );
}
