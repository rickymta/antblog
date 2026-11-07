import 'package:test/test.dart';
import 'package:toi_noi_53/release_plan.dart' show AppKind;
import 'package:toi_noi_55/toi_noi_55.dart';

import 'support.dart';

void main() {
  group('Commit.parseLine', () {
    test('tách sha và subject theo TAB', () {
      final c = Commit.parseLine('a1b2c3d\tfeat: thêm A');
      expect(c.sha, 'a1b2c3d');
      expect(c.subject, 'feat: thêm A');
    });

    test('subject có TAB giữ nguyên phần sau TAB đầu tiên', () {
      expect(Commit.parseLine('abc\tfix: a\tb').subject, 'fix: a\tb');
    });

    test('dòng không có TAB là subject thuần', () {
      final c = Commit.parseLine('fix: không có sha');
      expect(c.sha, isEmpty);
      expect(c.subject, 'fix: không có sha');
    });

    test('dòng rỗng ném FormatException', () {
      expect(() => Commit.parseLine('   '), throwsFormatException);
    });
  });

  group('classifyCommit: lọc commit nội bộ', () {
    NoteItem? classify(String subject, {AppKind app = AppKind.customer}) =>
        classifyCommit(Commit(subject), app: app);

    test('giữ feat, fix, perf và gán đúng nhóm', () {
      expect(classify('feat: a')!.section, NoteSection.feature);
      expect(classify('fix: a')!.section, NoteSection.fix);
      expect(classify('perf: a')!.section, NoteSection.performance);
    });

    test('bỏ mọi kiểu còn lại', () {
      for (final type in [
        'chore',
        'ci',
        'build',
        'test',
        'docs',
        'refactor',
        'style',
        'revert',
      ]) {
        expect(classify('$type: việc nội bộ'), isNull, reason: type);
      }
    });

    test('bỏ commit merge, revert thuần và commit không theo quy ước', () {
      expect(classify('Merge branch main into x'), isNull);
      expect(classify('Revert "feat: a"'), isNull);
      expect(classify('sửa linh tinh'), isNull);
      expect(classify('feat thiếu dấu hai chấm'), isNull);
    });

    test('kiểu viết hoa và có dấu ! vẫn nhận', () {
      expect(classify('Feat: a')!.section, NoteSection.feature);
      expect(
        classify('feat!: đổi cách đặt chuyến')!.text,
        'Đổi cách đặt chuyến',
      );
      expect(classify('fix(core)!: a')!.section, NoteSection.fix);
    });

    test('thẻ [skip-notes] và [internal] loại cả feat lẫn fix', () {
      expect(classify('feat: cờ thử nghiệm [skip-notes]'), isNull);
      expect(classify('fix: dọn log [Internal]'), isNull);
    });

    test('scope là app kia bị loại, scope khác hoặc không scope được giữ', () {
      expect(classify('feat(driver): a'), isNull);
      expect(classify('feat(driver): a', app: AppKind.driver), isNotNull);
      expect(classify('feat(customer): a', app: AppKind.driver), isNull);
      expect(classify('feat(map): a'), isNotNull);
      expect(classify('feat: a', app: AppKind.driver), isNotNull);
    });

    test('làm sạch: bỏ PR, thẻ, dấu chấm cuối; viết hoa chữ đầu có dấu', () {
      expect(
        classify('fix: ủng hộ chuyến đi xa (#12).')!.text,
        'Ủng hộ chuyến đi xa',
      );
      expect(classify('fix: đổi màu nút [wip] (#3)')!.text, 'Đổi màu nút');
      expect(
        classify('fix:    nhiều    khoảng   trắng ')!.text,
        'Nhiều khoảng trắng',
      );
      expect(classify('fix: đếm lại...')!.text, 'Đếm lại');
    });

    test('mô tả rỗng sau khi làm sạch bị coi là nội bộ', () {
      expect(classify('fix: [wip]'), isNull);
      expect(classify('fix: (#9)'), isNull);
    });
  });

  group('ReleaseNotes: nội dung', () {
    test(
      'mẫu: giữ 4 dòng, bỏ 1 dòng trùng và 8 commit không dành cho tester',
      () {
        final notes = notesFor(sampleCommits());
        expect(notes.included, 4);
        expect(notes.omitted, 0);
        // 13 commit = 4 giữ + 1 trùng + 8 bỏ (chore, ci, test, refactor, merge,
        // commit không theo quy ước, thẻ skip-notes, commit của app tài xế).
        expect(notes.skipped, 8);
        expect(
          notes.text,
          '''
Tới Nơi khách · staging 1.4.0 (build 42)

Tính năng mới
- Thêm đếm ngược khi tìm tài xế

Sửa lỗi
- Cước 2,001 km hiển thị đúng $fare2001
- Sửa nút gọi lại bị ẩn sau bàn phím

Hiệu năng
- Giảm số lần vẽ lại polyline'''
              .replaceFirst('\n\nHiệu năng', '\n\nHiệu năng'),
        );
      },
    );

    test(
      'cước trong notes là số của calculateFare: 2,001 km là 12.005 VND',
      () {
        expect(fare2001, '12.005 VND');
        expect(notesFor(sampleCommits()).text, contains('12.005 VND'));
      },
    );

    test('sinh cho app tài xế: lấy commit driver, bỏ commit customer', () {
      final text = notesFor(sampleCommits(), app: AppKind.driver).text;
      expect(text, startsWith('Tới Nơi tài xế · staging 1.4.0 (build 42)'));
      expect(text, contains('Nhận cuốc bằng một chạm'));
      expect(text, isNot(contains('đếm ngược')));
    });

    test('thứ tự nhóm cố định dù commit đảo lộn', () {
      final text = notesFor([
        const Commit('perf: c'),
        const Commit('fix: b'),
        const Commit('feat: a'),
      ]).text;
      expect(text.indexOf('Tính năng mới'), lessThan(text.indexOf('Sửa lỗi')));
      expect(text.indexOf('Sửa lỗi'), lessThan(text.indexOf('Hiệu năng')));
    });

    test('không có thay đổi nào cho người dùng thì nói rõ, không để trống', () {
      final notes = notesFor([const Commit('chore: a'), const Commit('ci: b')]);
      expect(notes.included, 0);
      expect(notes.skipped, 2);
      expect(notes.text, contains('chỉ cải tiến nội bộ'));
    });

    test('danh sách commit rỗng cũng cho notes hợp lệ', () {
      expect(notesFor(const []).text, contains('chỉ cải tiến nội bộ'));
    });

    test('SHA và tên tác giả không bao giờ lọt vào notes', () {
      final text = notesFor(sampleCommits()).text;
      for (final c in sampleCommits()) {
        expect(text, isNot(contains(c.sha)), reason: c.sha);
      }
    });
  });

  group('ReleaseNotes: cắt độ dài', () {
    final many = [
      for (var i = 1; i <= 40; i++) Commit('feat: tính năng số $i của Tới Nơi'),
    ];

    test('không bao giờ vượt maxLength, tính theo ký tự Unicode', () {
      for (final max in [100, 150, 200, 333, 500]) {
        final notes = notesFor(many, maxLength: max);
        expect(notes.length, lessThanOrEqualTo(max), reason: 'max=$max');
        expect(notes.text.runes.length, notes.length);
      }
    });

    test('bỏ cả dòng từ cuối và nói rõ còn bao nhiêu thay đổi khác', () {
      final notes = notesFor(many, maxLength: 200);
      expect(notes.included, lessThan(40));
      expect(notes.omitted, 40 - notes.included);
      expect(notes.text, endsWith('… và ${notes.omitted} thay đổi khác'));
      // Giữ nguyên các dòng đầu, không cắt giữa dòng.
      expect(notes.text, contains('- Tính năng số 1 của Tới Nơi\n'));
    });

    test('đủ chỗ thì không có dòng lược', () {
      final notes = notesFor(many.take(3), maxLength: 500);
      expect(notes.omitted, 0);
      expect(notes.text, isNot(contains('thay đổi khác')));
    });

    test('một dòng quá dài bị cắt tại ranh giới từ, kết thúc bằng …', () {
      final long = 'từ ' * 100;
      final notes = notesFor([Commit('feat: $long')], maxLength: 120);
      expect(notes.length, lessThanOrEqualTo(120));
      expect(notes.text, endsWith('từ…'));
      expect(notes.included, 1);
    });

    test('dòng quá dài kèm các dòng khác: có cả cắt dòng lẫn dòng lược', () {
      final notes = notesFor([
        Commit('feat: ${'dài ' * 80}'),
        const Commit('fix: b'),
        const Commit('fix: c'),
      ], maxLength: 130);
      expect(notes.length, lessThanOrEqualTo(130));
      expect(notes.text, endsWith('… và 2 thay đổi khác'));
    });

    test('maxLength nhỏ hơn mức tối thiểu bị từ chối', () {
      expect(
        () => notesFor(many, maxLength: minNotesLength - 1),
        throwsArgumentError,
      );
    });

    test('maxLength quá nhỏ so với tiêu đề của bản dài bị từ chối', () {
      expect(
        () => ReleaseNotes.fromCommits(
          [Commit('feat: ${'dài ' * 80}')],
          app: AppKind.driver,
          version: '${'9' * 30}.${'9' * 30}.${'9' * 30}',
          build: 2100000000,
          maxLength: minNotesLength,
        ),
        throwsArgumentError,
      );
    });

    test('version và build sai bị từ chối', () {
      for (final bad in ['1.4', 'v1.4.0', '1.4.0+42', '']) {
        expect(
          () => ReleaseNotes.fromCommits(
            const [],
            app: AppKind.customer,
            version: bad,
            build: 1,
          ),
          throwsArgumentError,
          reason: bad,
        );
      }
      expect(
        () => ReleaseNotes.fromCommits(
          const [],
          app: AppKind.customer,
          version: '1.4.0',
          build: 0,
        ),
        throwsArgumentError,
      );
    });
  });

  group('truncateAtWord', () {
    test('ngắn hơn giới hạn thì giữ nguyên', () {
      expect(truncateAtWord('chào bạn', 8), 'chào bạn');
    });

    test('cắt tại khoảng trắng và thêm …', () {
      expect(truncateAtWord('một hai ba bốn năm', 12), 'một hai ba…');
    });

    test('không tìm được ranh giới từ hợp lý thì cắt cứng', () {
      expect(truncateAtWord('abcdefghijklmnop', 6), 'abcde…');
    });

    test('không cắt giữa cặp surrogate của emoji', () {
      bool hasLoneSurrogate(String s) {
        final units = s.codeUnits;
        for (var i = 0; i < units.length; i++) {
          final u = units[i];
          if (u >= 0xD800 && u <= 0xDBFF) {
            if (i + 1 >= units.length ||
                units[i + 1] < 0xDC00 ||
                units[i + 1] > 0xDFFF) {
              return true;
            }
            i++;
          } else if (u >= 0xDC00 && u <= 0xDFFF) {
            return true;
          }
        }
        return false;
      }

      const text = 'xe 🚗🚗🚗🚗🚗🚗';
      for (var limit = 2; limit <= 10; limit++) {
        final out = truncateAtWord(text, limit);
        expect(hasLoneSurrogate(out), isFalse, reason: 'limit=$limit');
        expect(out.runes.length, lessThanOrEqualTo(limit));
      }
      // Phép thử có tác dụng: cắt theo code unit sẽ làm vỡ chuỗi này.
      expect(hasLoneSurrogate(text.substring(0, 4)), isTrue);
    });

    test('chuỗi NFD không để lại dấu kết hợp mồ côi', () {
      // "ê" ở dạng NFD là e + U+0302; lặp để có điểm cắt giữa chữ và dấu.
      final text = List.filled(20, 'e\u0302').join();
      for (var limit = 2; limit <= 12; limit++) {
        final out = truncateAtWord(text, limit);
        final runes = out.runes.toList();
        expect(runes.last, 0x2026, reason: 'limit=$limit');
        // Ký tự ngay trước … phải là dấu kết hợp (cụm trọn vẹn), không phải
        // chữ e trơ trọi bị tách khỏi dấu.
        if (runes.length > 1) {
          expect(runes[runes.length - 2], 0x0302, reason: 'limit=$limit');
        }
        expect(runes.length, lessThanOrEqualTo(limit));
      }
    });

    test('limit nhỏ hơn 1 bị từ chối', () {
      expect(() => truncateAtWord('a', 0), throwsArgumentError);
    });
  });
}
