import 'model.dart';

/// Hai trạng thái kết thúc, là const Set: bất biến và dùng chung một bản.
const terminalStatuses = <TripStatus>{
  TripStatus.completed,
  TripStatus.cancelled,
};

/// collection-if chọn phần tử theo điều kiện; spread `...` chèn cả list.
List<String> homeTabs({
  required bool isDriverApp,
  List<String> extra = const [],
}) => ['Chuyến', if (isDriverApp) 'Thu nhập' else 'Ưu đãi', ...extra];

/// collection-for dựng Map: mọi trạng thái bắt đầu từ 0 chuyến.
Map<TripStatus, int> emptyStatusCounts() => {
  for (final status in TripStatus.values) status: 0,
};
