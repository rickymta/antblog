/// Nguồn thời gian có thể thay trong test.
typedef Clock = DateTime Function();

/// Đồng hồ thật: luôn trả giờ UTC, đúng quy ước của `Trip.requestedAt`.
DateTime systemClock() => DateTime.now().toUtc();
