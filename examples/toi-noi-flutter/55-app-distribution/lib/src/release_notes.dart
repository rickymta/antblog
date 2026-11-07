import 'package:toi_noi_53/release_plan.dart' show AppKind;

/// Độ dài tối đa mặc định của release notes, tính bằng ký tự Unicode (code point).
///
/// Đây là lựa chọn biên tập để ghi chú vừa một màn hình điện thoại của tester,
/// không phải giới hạn của Firebase: tài liệu REST của App Distribution không
/// nêu giới hạn độ dài cho `ReleaseNotes.text` (kiểm tại 11/2026).
const int defaultMaxNotesLength = 500;

/// Độ dài tối thiểu chấp nhận được cho `maxLength`: đủ chứa tiêu đề và một dòng.
const int minNotesLength = 100;

/// Một commit lấy từ `git log --pretty=format:'%h%x09%s'`.
final class Commit {
  const Commit(this.subject, {this.sha = ''});

  /// Đọc một dòng dạng `sha<TAB>subject`. Dòng không có TAB được coi là chỉ có
  /// subject. Ném [FormatException] khi dòng rỗng.
  factory Commit.parseLine(String line) {
    final trimmed = line.trim();
    if (trimmed.isEmpty) {
      throw const FormatException('dòng commit rỗng');
    }
    final tab = trimmed.indexOf('\t');
    if (tab < 0) return Commit(trimmed);
    return Commit(
      trimmed.substring(tab + 1).trim(),
      sha: trimmed.substring(0, tab),
    );
  }

  final String subject;

  /// Chỉ để đối chiếu log của CI. Không bao giờ nằm trong release notes.
  final String sha;
}

/// Ba nhóm commit được đưa vào release notes, theo thứ tự hiển thị.
enum NoteSection {
  feature('Tính năng mới', 'feat'),
  fix('Sửa lỗi', 'fix'),
  performance('Hiệu năng', 'perf');

  const NoteSection(this.heading, this.type);

  /// Tiêu đề tiếng Việt hiện cho tester.
  final String heading;

  /// Kiểu commit theo Conventional Commits tương ứng.
  final String type;
}

/// Một dòng đã được làm sạch, sẵn sàng đưa vào notes.
final class NoteItem {
  const NoteItem(this.section, this.text);

  final NoteSection section;
  final String text;
}

/// Kết quả phân loại một commit: [item] có nghĩa là hiện cho tester, `null` là
/// commit nội bộ.
NoteItem? classifyCommit(Commit commit, {required AppKind app}) {
  final subject = commit.subject.trim();
  if (_internalMarker.hasMatch(subject)) return null;
  final match = _conventional.firstMatch(subject);
  if (match == null) return null;

  final type = match.group(1)!.toLowerCase();
  final scope = match.group(2)?.trim().toLowerCase();
  final section = NoteSection.values.where((s) => s.type == type).firstOrNull;
  if (section == null) return null;

  // Commit của app kia không liên quan tới tester của app này.
  final otherApp = AppKind.values.firstWhere((a) => a != app);
  if (scope == otherApp.name) return null;

  final text = _cleanDescription(match.group(3)!);
  if (text.isEmpty) return null;
  return NoteItem(section, text);
}

/// Release notes tiếng Việt cho tester, sinh từ danh sách commit.
///
/// Quy tắc, theo thứ tự:
/// 1. Chỉ giữ commit `feat`, `fix`, `perf`. Mọi kiểu khác (`chore`, `ci`,
///    `build`, `test`, `docs`, `refactor`, `style`), commit merge/revert và
///    commit không theo Conventional Commits là commit nội bộ và bị bỏ.
/// 2. Commit có thẻ `[skip-notes]` hoặc `[internal]` bị bỏ dù là `feat`/`fix`.
/// 3. Commit có scope là tên app kia (`driver` khi sinh cho app khách) bị bỏ.
/// 4. Bỏ phần `type(scope):`, số PR `(#123)` và dấu chấm cuối; viết hoa chữ đầu;
///    bỏ dòng trùng.
/// 5. Cắt cho vừa `maxLength` ký tự: bỏ bớt cả dòng từ cuối, thay bằng một dòng
///    "… và N thay đổi khác". Nếu ngay một dòng đã quá dài thì cắt dòng đó tại
///    ranh giới từ và thêm "…".
///
/// SHA và tác giả không bao giờ xuất hiện trong kết quả.
final class ReleaseNotes {
  ReleaseNotes._({
    required this.text,
    required this.included,
    required this.omitted,
    required this.skipped,
  });

  /// Sinh release notes cho [app], flavor staging, bản [version] (build [build]).
  ///
  /// Ném [ArgumentError] khi [maxLength] nhỏ hơn [minNotesLength], khi [build]
  /// không dương, hoặc khi [version] không có dạng `x.y.z`.
  factory ReleaseNotes.fromCommits(
    Iterable<Commit> commits, {
    required AppKind app,
    required String version,
    required int build,
    int maxLength = defaultMaxNotesLength,
  }) {
    if (maxLength < minNotesLength) {
      throw ArgumentError.value(
        maxLength,
        'maxLength',
        'phải từ $minNotesLength ký tự trở lên',
      );
    }
    if (build < 1) {
      throw ArgumentError.value(build, 'build', 'phải là số nguyên dương');
    }
    if (!_semver.hasMatch(version)) {
      throw ArgumentError.value(version, 'version', 'phải có dạng x.y.z');
    }

    final seen = <String>{};
    final items = <NoteItem>[];
    var internal = 0;
    for (final commit in commits) {
      final item = classifyCommit(commit, app: app);
      if (item == null) {
        internal++;
      } else if (seen.add(item.text.toLowerCase())) {
        items.add(item);
      }
    }
    // Giữ thứ tự nhóm cố định; trong nhóm giữ thứ tự của `git log`.
    final ordered = [
      for (final section in NoteSection.values)
        ...items.where((i) => i.section == section),
    ];

    final header =
        'Tới Nơi ${_appLabel(app)} · staging $version (build $build)';
    final fit = _fit(header, ordered, maxLength);
    return ReleaseNotes._(
      text: fit.text,
      included: fit.shown,
      omitted: ordered.length - fit.shown,
      skipped: internal,
    );
  }

  /// Nội dung ghi vào file cho `--release-notes-file`.
  final String text;

  /// Số dòng hiện trong [text] (kể cả dòng bị cắt ngắn).
  final int included;

  /// Số thay đổi bị lược vì hết chỗ.
  final int omitted;

  /// Số commit bị bỏ: nội bộ, thẻ bỏ qua, hoặc thuộc app còn lại.
  final int skipped;

  /// Số ký tự Unicode của [text].
  int get length => text.runes.length;

  @override
  String toString() => text;
}

String _appLabel(AppKind app) => switch (app) {
  AppKind.customer => 'khách',
  AppKind.driver => 'tài xế',
};

const String _emptyBody =
    'Không có thay đổi nào dành cho người dùng, chỉ cải tiến nội bộ.';

final RegExp _semver = RegExp(r'^\d+\.\d+\.\d+$');
final RegExp _internalMarker = RegExp(
  r'\[(skip-notes|internal)\]',
  caseSensitive: false,
);
final RegExp _conventional = RegExp(r'^([A-Za-z]+)(?:\(([^)]*)\))?!?:\s*(.+)$');
final RegExp _prSuffix = RegExp(r'\s*\(#\d+\)\s*$');
final RegExp _tags = RegExp(r'\s*\[[^\]]*\]');

String _cleanDescription(String raw) {
  var text = raw.replaceAll(_tags, '').replaceAll(RegExp(r'\s+'), ' ').trim();
  // Dấu chấm cuối có thể đứng trước hoặc sau số PR: bỏ lần lượt tới khi hết.
  String previous;
  do {
    previous = text;
    text = text
        .replaceAll(_prSuffix, '')
        .replaceAll(RegExp(r'\.+$'), '')
        .trim();
  } while (text != previous);
  if (text.isEmpty) return text;
  final first = String.fromCharCode(text.runes.first);
  return first.toUpperCase() + text.substring(first.length);
}

/// Dựng text cho [shown] dòng đầu của [items]; thêm dòng "… và N thay đổi khác"
/// khi còn dòng bị lược.
String _render(String header, List<NoteItem> items, int shown) {
  if (items.isEmpty) return '$header\n\n$_emptyBody';
  final buffer = StringBuffer(header);
  NoteSection? current;
  for (final item in items.take(shown)) {
    if (item.section != current) {
      current = item.section;
      buffer.write('\n\n${item.section.heading}');
    }
    buffer.write('\n- ${item.text}');
  }
  final rest = items.length - shown;
  if (rest > 0) {
    buffer.write(shown == 0 ? '\n\n' : '\n');
    buffer.write('… và $rest thay đổi khác');
  }
  return buffer.toString();
}

({String text, int shown}) _fit(
  String header,
  List<NoteItem> items,
  int maxLength,
) {
  for (var shown = items.length; shown >= 1; shown--) {
    final text = _render(header, items, shown);
    if (text.runes.length <= maxLength) return (text: text, shown: shown);
  }
  if (items.isEmpty) {
    return (text: _render(header, items, 0), shown: 0);
  }
  // Ngay cả một dòng cũng không vừa: cắt dòng đầu tiên cho vừa phần còn lại.
  final first = items.first;
  final rest = items.length - 1;
  final overflow = rest > 0 ? '\n… và $rest thay đổi khác' : '';
  final prefix = '$header\n\n${first.section.heading}\n- ';
  final budget = maxLength - prefix.runes.length - overflow.runes.length;
  if (budget < 10) {
    throw ArgumentError.value(
      maxLength,
      'maxLength',
      'quá nhỏ so với tiêu đề của bản này',
    );
  }
  final cut = truncateAtWord(first.text, budget);
  return (text: '$prefix$cut$overflow', shown: 1);
}

/// Cắt [text] còn tối đa [limit] ký tự Unicode, thêm "…" khi có cắt.
///
/// Không cắt giữa cặp surrogate (emoji), không để lại dấu kết hợp mồ côi ở
/// chuỗi dạng NFD, và ưu tiên cắt tại khoảng trắng.
String truncateAtWord(String text, int limit) {
  if (limit < 1) {
    throw ArgumentError.value(limit, 'limit', 'phải từ 1 trở lên');
  }
  final runes = text.runes.toList();
  if (runes.length <= limit) return text;

  var end = limit - 1; // chừa chỗ cho "…"
  // Lùi nếu ký tự ngay sau điểm cắt là dấu kết hợp: không tách dấu khỏi chữ.
  while (end > 0 && _isCombining(runes[end])) {
    end--;
  }
  final space = runes.lastIndexOf(0x20, end);
  // Chỉ cắt theo từ khi không làm mất quá nửa phần còn lại.
  if (space > end ~/ 2) end = space;
  final kept = String.fromCharCodes(runes.sublist(0, end)).trimRight();
  return '$kept…';
}

bool _isCombining(int rune) =>
    (rune >= 0x0300 && rune <= 0x036F) ||
    rune == 0x200D ||
    (rune >= 0xFE00 && rune <= 0xFE0F);
