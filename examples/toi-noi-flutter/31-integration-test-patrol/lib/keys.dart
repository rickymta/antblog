import 'package:flutter/foundation.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

/// Một nơi duy nhất đặt tên cho các widget mà test cần tìm. App dùng chúng để
/// gắn `key:`, test dùng chúng để `find.byKey`: đổi tên ở đây thì cả hai cùng
/// đổi, không có chuỗi chép tay ở hai nơi.
///
/// Tên là một từ không dấu cách và không gạch nối để dùng được với cú pháp
/// `$(#distanceField)` của Patrol (`#tên` tương đương `Key('tên')`).
abstract final class AppKeys {
  static const distanceField = Key('distanceField');
  static const quoteButton = Key('quoteButton');
  static const fareText = Key('fareText');
  static const bookButton = Key('bookButton');
  static const errorText = Key('errorText');

  static const tripIdText = Key('tripIdText');
  static const tripStatusText = Key('tripStatusText');
  static const driverText = Key('driverText');
  static const enableNotificationsButton = Key('enableNotificationsButton');
  static const notificationStatusText = Key('notificationStatusText');

  static Key service(ServiceKind kind) => Key('service_${kind.name}');
}
