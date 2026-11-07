/// Kênh thông báo Android cho cập nhật chuyến. Id này phải khớp giá trị
/// `default_notification_channel_id` trong AndroidManifest và `channel_id`
/// trong tin backend gửi; `native_config_test.dart` kiểm điều đó.
const tripUpdatesChannelId = 'trip_updates';
const tripUpdatesChannelName = 'Cập nhật chuyến đi';
const tripUpdatesChannelDescription =
    'Tài xế nhận chuyến, đang tới, đã đến điểm đón.';
