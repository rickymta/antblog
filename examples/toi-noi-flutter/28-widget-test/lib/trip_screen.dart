import 'package:flutter/material.dart';
import 'package:toi_noi_core/toi_noi_core.dart';
import 'package:toi_noi_ui/toi_noi_ui.dart';

/// Màn hình chuyến sau khi đặt: bài này chỉ cần mã chuyến và trạng thái.
class TripScreen extends StatelessWidget {
  const TripScreen({required this.id, super.key});

  final String id;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: Text('Chuyến $id')),
      body: const Padding(
        padding: EdgeInsets.all(ToiNoiSpacing.md),
        child: TripStatusChip(status: TripStatus.requested),
      ),
    );
  }
}
