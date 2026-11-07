import 'dart:convert';
import 'dart:io';
import 'dart:math';

import 'location_sample.dart';

/// Nơi bộ đệm được lưu để sống qua việc app bị đóng và để chia sẻ giữa các
/// isolate.
///
/// Task của `workmanager` chạy trong isolate riêng với bộ nhớ riêng: biến
/// hay `LocationBuffer` của giao diện không thấy được từ đó. Chỉ dữ liệu ghi
/// ra đĩa mới đi qua được.
abstract interface class BufferStore {
  Future<List<LocationSample>> load();
  Future<void> save(List<LocationSample> samples);
}

/// Bản trong bộ nhớ, dùng cho test.
final class MemoryBufferStore implements BufferStore {
  MemoryBufferStore([Iterable<LocationSample> initial = const []])
    : _saved = List.of(initial);

  List<LocationSample> _saved;
  int saveCount = 0;

  @override
  Future<List<LocationSample>> load() async => List.of(_saved);

  @override
  Future<void> save(List<LocationSample> samples) async {
    _saved = List.of(samples);
    saveCount++;
  }
}

/// Lưu cả bộ đệm thành một file JSON.
final class FileBufferStore implements BufferStore {
  FileBufferStore(this.file);

  final File file;

  /// File hỏng hoặc sai định dạng cho danh sách rỗng: bộ đệm chỉ là bản nháp
  /// của dữ liệu tạm, mất nó còn hơn để task nền ném lỗi mãi.
  @override
  Future<List<LocationSample>> load() async {
    if (!await file.exists()) return const [];
    try {
      final decoded = jsonDecode(await file.readAsString());
      if (decoded is! List<Object?>) {
        throw const FormatException('Bộ đệm phải là một mảng JSON');
      }
      return decoded.map(LocationSample.fromJson).toList();
    } on FormatException {
      return const [];
    }
  }

  /// Ghi ra file tạm **riêng của lần ghi này** rồi đổi tên: người đọc thấy bản
  /// cũ hoặc bản mới nguyên vẹn, không bao giờ thấy nửa chừng. Tên tạm riêng vì
  /// nhiều isolate có thể ghi cùng lúc; dùng chung một tên thì lần này đổi tên
  /// mất file của lần kia. Hai lần ghi đồng thời không được hợp nhất: bản ghi
  /// sau cùng thắng và bản kia mất.
  @override
  Future<void> save(List<LocationSample> samples) async {
    final tmp = File(
      '${file.path}.${pid}_${DateTime.now().microsecondsSinceEpoch}_'
      '${_random.nextInt(1 << 30)}.tmp',
    );
    try {
      await tmp.writeAsString(
        jsonEncode(samples.map((s) => s.toJson()).toList()),
        flush: true,
      );
      await tmp.rename(file.path);
    } on FileSystemException {
      if (tmp.existsSync()) tmp.deleteSync();
      rethrow;
    }
  }

  static final Random _random = Random.secure();
}
