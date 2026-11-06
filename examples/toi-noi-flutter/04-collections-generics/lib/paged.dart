/// Một trang kết quả của kiểu bất kỳ. `page` đếm từ 1.
class Paged<T> {
  const Paged({
    required this.items,
    required this.page,
    required this.pageSize,
    required this.total,
  });

  final List<T> items;
  final int page;
  final int pageSize;

  /// Tổng số phần tử của toàn bộ danh sách, không phải của trang này.
  final int total;

  /// Còn trang sau khi số phần tử đã lấy (tới hết trang này) chưa đủ total.
  bool get hasNext => page * pageSize < total;

  /// Đổi kiểu phần tử, giữ nguyên thông tin phân trang.
  Paged<R> map<R>(R Function(T item) transform) => Paged<R>(
    items: List.unmodifiable(items.map(transform)),
    page: page,
    pageSize: pageSize,
    total: total,
  );

  @override
  String toString() => 'Paged(page $page, ${items.length}/$total items)';
}

/// Cắt một trang từ danh sách đầy đủ. Trang vượt cuối cho items rỗng.
Paged<T> paginate<T>(List<T> all, {required int page, required int pageSize}) {
  if (page < 1) throw ArgumentError.value(page, 'page', 'must be >= 1');
  if (pageSize < 1) {
    throw ArgumentError.value(pageSize, 'pageSize', 'must be >= 1');
  }
  final items = all.skip((page - 1) * pageSize).take(pageSize);
  return Paged<T>(
    items: List.unmodifiable(items),
    page: page,
    pageSize: pageSize,
    total: all.length,
  );
}

/// Phần tử có khoá lớn nhất. `K extends Comparable<K>` ràng buộc kiểu khoá:
/// compareTo có sẵn, không cần ép kiểu hay dynamic. String, DateTime, num dùng
/// được. Lưu ý `int` là `Comparable<num>`, nên viết tường minh `K = int` bị
/// từ chối; để Dart suy luận (K = num) hoặc dùng khoá String/DateTime.
T? highestBy<T, K extends Comparable<K>>(
  Iterable<T> items,
  K Function(T item) keyOf,
) {
  T? best;
  K? bestKey;
  for (final item in items) {
    final key = keyOf(item);
    if (bestKey == null || key.compareTo(bestKey) > 0) {
      best = item;
      bestKey = key;
    }
  }
  return best;
}
