import 'package:test/test.dart';
import 'package:toi_noi_04/paged.dart';

void main() {
  final all = List.generate(8, (i) => i + 1); // 1..8

  group('paginate', () {
    test('trang đầu đầy, còn trang sau', () {
      final page = paginate(all, page: 1, pageSize: 3);
      expect(page.items, [1, 2, 3]);
      expect(page.total, 8);
      expect(page.hasNext, isTrue);
    });

    test('trang cuối lẻ: 8 phần tử, cỡ 3, trang 3 chỉ có 2', () {
      final page = paginate(all, page: 3, pageSize: 3);
      expect(page.items, [7, 8]);
      expect(page.hasNext, isFalse);
    });

    test('hasNext sai khi trang cuối vừa khít', () {
      final page = paginate(all, page: 2, pageSize: 4);
      expect(page.items, [5, 6, 7, 8]);
      expect(page.hasNext, isFalse);
    });

    test('trang vượt cuối cho items rỗng, danh sách rỗng cũng vậy', () {
      expect(paginate(all, page: 9, pageSize: 3).items, isEmpty);
      expect(paginate(all, page: 9, pageSize: 3).hasNext, isFalse);
      final empty = paginate(<int>[], page: 1, pageSize: 3);
      expect(empty.items, isEmpty);
      expect(empty.total, 0);
      expect(empty.hasNext, isFalse);
    });

    test('page hoặc pageSize không hợp lệ ném ArgumentError', () {
      expect(() => paginate(all, page: 0, pageSize: 3), throwsArgumentError);
      expect(() => paginate(all, page: 1, pageSize: 0), throwsArgumentError);
    });

    test('items không sửa được, và không đổi theo list gốc sau khi cắt', () {
      final source = [1, 2, 3, 4];
      final page = paginate(source, page: 1, pageSize: 2);
      expect(() => page.items.add(9), throwsUnsupportedError);
      source[0] = 99;
      expect(page.items, [1, 2]);
    });
  });

  group('Paged.map', () {
    test('đổi kiểu phần tử, giữ nguyên page, pageSize, total, hasNext', () {
      final page = paginate(all, page: 2, pageSize: 3);
      final labels = page.map((n) => 'chuyến $n');
      expect(labels, isA<Paged<String>>());
      expect(labels.items, ['chuyến 4', 'chuyến 5', 'chuyến 6']);
      expect(labels.page, 2);
      expect(labels.pageSize, 3);
      expect(labels.total, 8);
      expect(labels.hasNext, page.hasNext);
    });
  });

  group('highestBy', () {
    test('chọn phần tử có khoá lớn nhất; rỗng trả null', () {
      final words = ['xe', 'xe máy', 'ô tô'];
      expect(highestBy(words, (w) => w.length), 'xe máy');
      expect(highestBy(<String>[], (w) => w.length), isNull);
    });

    test('khoá bằng nhau: giữ phần tử gặp trước', () {
      expect(highestBy(['aa', 'bb'], (w) => w.length), 'aa');
    });
  });
}
