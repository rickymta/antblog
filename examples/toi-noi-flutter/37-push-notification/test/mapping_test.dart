import 'package:firebase_messaging/firebase_messaging.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_37/firebase_push_source.dart';
import 'package:toi_noi_37/push_message.dart';

void main() {
  group('toPushMessage', () {
    test('lấy id, tiêu đề, nội dung và data', () {
      final message = toPushMessage(
        const RemoteMessage(
          messageId: 'm-1',
          notification: RemoteNotification(title: 'Tiêu đề', body: 'Nội dung'),
          data: {'type': 'trip_update', 'tripId': 'T-1'},
        ),
      );
      expect(message.id, 'm-1');
      expect(message.title, 'Tiêu đề');
      expect(message.body, 'Nội dung');
      expect(message.data, {'type': 'trip_update', 'tripId': 'T-1'});
    });

    test('tin chỉ có data thì tiêu đề và nội dung là null', () {
      final message = toPushMessage(
        const RemoteMessage(data: {'type': 'trip_update'}),
      );
      expect(message.title, isNull);
      expect(message.body, isNull);
    });
  });

  group('toPushPermission', () {
    test('mỗi trạng thái của plugin có một nghĩa', () {
      expect(
        {for (final s in AuthorizationStatus.values) s: toPushPermission(s)},
        {
          AuthorizationStatus.authorized: PushPermission.granted,
          AuthorizationStatus.provisional: PushPermission.granted,
          AuthorizationStatus.notDetermined: PushPermission.notAsked,
          AuthorizationStatus.denied: PushPermission.denied,
          AuthorizationStatus.deniedPermanently: PushPermission.deniedForever,
        },
      );
    });
  });
}
