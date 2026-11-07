import 'dart:io';
import 'dart:isolate';

import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_38/buffer_store.dart';
import 'package:toi_noi_38/location_buffer.dart';
import 'package:toi_noi_38/location_sample.dart';

import 'test_helpers.dart';

/// Bộ đệm "của giao diện": một biến cấp cao nhất sống trong bộ nhớ của isolate
/// này. Isolate khác nhận một bản khởi tạo riêng, trống.
final LocationBuffer processBuffer = LocationBuffer();

void main() {
  late Directory dir;
  late File file;

  setUp(() {
    dir = Directory.systemTemp.createTempSync('toi_noi_38_store_');
    file = File('${dir.path}/pending_samples.json');
  });

  tearDown(() => dir.deleteSync(recursive: true));

  group('FileBufferStore', () {
    test('ghi rồi đọc lại đúng thứ tự', () async {
      final store = FileBufferStore(file);
      await store.save([sample(0), sample(1)]);
      expect(await store.load(), [sample(0), sample(1)]);
    });

    test('không để lại file tạm sau khi lưu', () async {
      await FileBufferStore(file).save([sample(0)]);
      expect(File('${file.path}.tmp').existsSync(), isFalse);
    });

    test('file chưa có: danh sách rỗng', () async {
      expect(await FileBufferStore(file).load(), isEmpty);
    });

    test('file hỏng (JSON cụt): danh sách rỗng, không ném lỗi', () async {
      file.writeAsStringSync('[{"tripId": "T1", ');
      expect(await FileBufferStore(file).load(), isEmpty);
    });

    test('file sai hình dạng (object thay vì mảng): danh sách rỗng', () async {
      file.writeAsStringSync('{"a": 1}');
      expect(await FileBufferStore(file).load(), isEmpty);
    });

    test('một mẫu sai kiểu làm cả file bị coi là hỏng', () async {
      file.writeAsStringSync('[{"tripId": 5}]');
      expect(await FileBufferStore(file).load(), isEmpty);
    });
  });

  group('nhiều isolate ghi cùng lúc', () {
    test(
      '3 isolate x 100 lần ghi: không lỗi, file cuối vẫn đọc được',
      () async {
        final path = file.path;
        // Mỗi isolate ghi mẫu mang tripId riêng để biết bản nào thắng.
        final results = await Future.wait([
          for (final id in ['A', 'B', 'C'])
            Isolate.run(() async {
              final store = FileBufferStore(File(path));
              var failures = 0;
              for (var i = 0; i < 100; i++) {
                try {
                  await store.save([
                    LocationSample(
                      tripId: id,
                      latitude: 10,
                      longitude: 106,
                      accuracyMeters: 5,
                      capturedAt: DateTime.utc(2026, 11, 6, 8, 0, i),
                    ),
                  ]);
                } on FileSystemException {
                  failures++;
                }
              }
              return failures;
            }),
        ]);
        expect(results, [0, 0, 0]);

        final last = await FileBufferStore(file).load();
        expect(
          last,
          hasLength(1),
          reason: 'bản ghi sau cùng thắng, không hợp nhất',
        );
        expect(['A', 'B', 'C'], contains(last.single.tripId));
        final leftovers = dir.listSync().where((e) => e.path.endsWith('.tmp'));
        expect(leftovers, isEmpty);
      },
    );
  });

  group('bộ nhớ không chia sẻ giữa isolate, đĩa thì có', () {
    test('biến trong bộ nhớ: isolate khác thấy bộ đệm trống', () async {
      processBuffer.add(sample(0));
      expect(processBuffer.length, 1);

      // Closure không bắt processBuffer; isolate mới tự khởi tạo biến cấp
      // cao nhất của nó từ đầu.
      final seenElsewhere = await Isolate.run(() => processBuffer.length);
      expect(seenElsewhere, 0);
    });

    test('file trên đĩa: isolate khác đọc được mẫu đã lưu', () async {
      await FileBufferStore(file).save([sample(0), sample(1)]);
      final path = file.path;

      final seenElsewhere = await Isolate.run(
        () async => (await FileBufferStore(File(path)).load()).length,
      );
      expect(seenElsewhere, 2);
    });
  });
}
