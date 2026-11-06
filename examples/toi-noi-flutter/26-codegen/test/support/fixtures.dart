/// Một chuyến hợp lệ đúng hình dạng JSON của fake API: 5 km, 25.500 VND (theo
/// `calculateFare(5)` của bảng giá minh hoạ), đang tìm tài xế.
Map<String, Object?> validTripJson() => <String, Object?>{
  'id': 'T-0001',
  'serviceKind': 'ride',
  'status': 'searching',
  'distanceMeters': 5000,
  'fare': 25500,
  'currency': 'VND',
  'driverId': null,
  'requestedAt': '2026-11-06T03:30:00.000Z',
  'version': 1,
};

Map<String, Object?> validQuoteJson() => <String, Object?>{
  'serviceKind': 'ride',
  'distanceMeters': 5000,
  'fare': 25500,
  'currency': 'VND',
};

/// Bản sao của [base] với các khoá trong [changes] được thay; giá trị
/// [absent] nghĩa là xoá khoá.
Map<String, Object?> changed(
  Map<String, Object?> base,
  Map<String, Object?> changes,
) {
  final copy = Map<String, Object?>.of(base);
  changes.forEach((key, value) {
    if (identical(value, absent)) {
      copy.remove(key);
    } else {
      copy[key] = value;
    }
  });
  return copy;
}

const absent = Object();

/// Bảng ca chung cho test so sánh: tên ca -> JSON đầu vào của `TripDto`.
/// Tám ca đầu hợp lệ, còn lại hợp đồng của `toi_noi_api` từ chối.
Map<String, Object?> tripCases() => <String, Object?>{
  // Hợp lệ.
  'chuyến hợp lệ': validTripJson(),
  'cước 25500.0 (decimal của .NET)': changed(validTripJson(), {
    'fare': 25500.0,
  }),
  'tên enum hoa thường lẫn lộn': changed(validTripJson(), {
    'serviceKind': 'RIDE',
    'status': 'PickedUp',
    'driverId': 'D-9',
  }),
  'độ lệch +07:00 đổi về UTC': changed(validTripJson(), {
    'requestedAt': '2026-11-06T10:30:00+07:00',
  }),
  'không có phần giây lẻ': changed(validTripJson(), {
    'requestedAt': '2026-11-06T03:30:00Z',
  }),
  'driverId vắng mặt': changed(validTripJson(), {'driverId': absent}),
  'khoá thừa bị bỏ qua': changed(validTripJson(), {'surge': 1.5}),
  'cước 0 và version 0': changed(validTripJson(), {'fare': 0, 'version': 0}),
  // Thiếu khoá.
  for (final key in validTripJson().keys.where((k) => k != 'driverId'))
    'thiếu $key': changed(validTripJson(), {key: absent}),
  // Sai kiểu hoặc sai giá trị.
  'id là số': changed(validTripJson(), {'id': 7}),
  'mét lẻ 5000.5': changed(validTripJson(), {'distanceMeters': 5000.5}),
  'mét là double 5000.0': changed(validTripJson(), {'distanceMeters': 5000.0}),
  'mét là chuỗi': changed(validTripJson(), {'distanceMeters': '5000'}),
  'mét âm': changed(validTripJson(), {'distanceMeters': -1}),
  'mét vượt 2^53': changed(validTripJson(), {
    'distanceMeters': 9007199254740993,
  }),
  'cước lẻ 25500.5': changed(validTripJson(), {'fare': 25500.5}),
  'cước âm': changed(validTripJson(), {'fare': -1}),
  'cước là chuỗi': changed(validTripJson(), {'fare': '25500'}),
  'cước null': changed(validTripJson(), {'fare': null}),
  'cước 1e300': changed(validTripJson(), {'fare': 1e300}),
  'tiền tệ USD': changed(validTripJson(), {'currency': 'USD'}),
  'tiền tệ null': changed(validTripJson(), {'currency': null}),
  'dịch vụ lạ': changed(validTripJson(), {'serviceKind': 'bike'}),
  'dịch vụ là số': changed(validTripJson(), {'serviceKind': 3}),
  'trạng thái lạ': changed(validTripJson(), {'status': 'flying'}),
  'driverId là số': changed(validTripJson(), {'driverId': 7}),
  'thời điểm thiếu múi giờ': changed(validTripJson(), {
    'requestedAt': '2026-11-06T03:30:00',
  }),
  'thời điểm tháng 13': changed(validTripJson(), {
    'requestedAt': '2026-13-45T03:30:00Z',
  }),
  'thời điểm 30/2': changed(validTripJson(), {
    'requestedAt': '2026-02-30T03:30:00Z',
  }),
  'thời điểm 25:61:61': changed(validTripJson(), {
    'requestedAt': '2026-11-06T25:61:61Z',
  }),
  'độ lệch +99:99': changed(validTripJson(), {
    'requestedAt': '2026-11-06T03:30:00+99:99',
  }),
  'chỉ có ngày': changed(validTripJson(), {'requestedAt': '2026-11-06'}),
  'thời điểm là số': changed(validTripJson(), {'requestedAt': 12345}),
  'thời điểm không phải ngày': changed(validTripJson(), {
    'requestedAt': 'hôm qua',
  }),
  'version âm': changed(validTripJson(), {'version': -1}),
  'version lẻ': changed(validTripJson(), {'version': 1.5}),
  // Không phải JSON object.
  'null': null,
  'mảng': <Object?>[],
  'chuỗi': 'T-0001',
  'số': 42,
};
