import 'dart:collection';

import 'location_sample.dart';

/// Bộ đệm vị trí chờ gửi khi mất mạng, **có giới hạn** theo cả số lượng lẫn
/// tuổi.
///
/// Vị trí cũ hơn vài phút không còn giúp khách thấy xe đang ở đâu, nên giữ vô
/// hạn chỉ tốn bộ nhớ, pin khi gửi bù và để lại dữ liệu vị trí không cần thiết
/// trên máy. Đầy thì bỏ mẫu cũ nhất, vì mẫu mới mô tả xe chính xác hơn.
final class LocationBuffer {
  /// Mặc định 120 mẫu và 10 phút: với một mẫu mỗi 5 giây thì hai giới hạn
  /// trùng nhau. Đây là số minh hoạ, không phải số đo.
  LocationBuffer({
    this.capacity = 120,
    this.maxAge = const Duration(minutes: 10),
  }) : assert(capacity > 0, 'capacity phải dương');

  final int capacity;
  final Duration maxAge;

  final Queue<LocationSample> _items = Queue<LocationSample>();
  int _droppedCount = 0;

  int get length => _items.length;
  bool get isEmpty => _items.isEmpty;

  /// Số mẫu đã bị bỏ vì bộ đệm đầy, để biết mạng mất lâu hay không.
  int get droppedCount => _droppedCount;

  /// Bản sao theo thứ tự cũ tới mới; sửa nó không ảnh hưởng bộ đệm.
  List<LocationSample> get items => List.unmodifiable(_items);

  void add(LocationSample sample) {
    if (_items.length >= capacity) {
      _items.removeFirst();
      _droppedCount++;
    }
    _items.addLast(sample);
  }

  /// Bỏ mẫu già hơn [maxAge] so với [now] và trả về số mẫu đã bỏ.
  int purgeExpired(DateTime now) {
    final before = _items.length;
    _items.removeWhere((s) => now.difference(s.capturedAt) > maxAge);
    return before - _items.length;
  }

  /// Tối đa [max] mẫu cũ nhất, chưa xoá khỏi bộ đệm: chỉ xoá (bằng
  /// [removeSent]) sau khi gửi xong.
  List<LocationSample> peek(int max) => _items.take(max).toList();

  /// Xoá đúng các mẫu trong [sent] theo danh tính đối tượng, không theo số
  /// lượng: trong lúc đang gửi, mẫu mới có thể đã đẩy một mẫu cũ ra khỏi bộ
  /// đệm đầy, và xoá "n mẫu đầu" lúc đó sẽ xoá nhầm mẫu chưa gửi.
  void removeSent(Iterable<LocationSample> sent) {
    final done = Set<LocationSample>.identity()..addAll(sent);
    _items.removeWhere(done.contains);
  }

  /// Nạp lại từ kho lưu trữ, vẫn tuân theo [capacity] (giữ các mẫu mới nhất).
  void replaceAll(Iterable<LocationSample> samples) {
    _items.clear();
    samples.forEach(add);
  }

  void clear() => _items.clear();
}
