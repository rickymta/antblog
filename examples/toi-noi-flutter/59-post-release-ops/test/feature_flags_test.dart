import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_59/feature_flags.dart';

import 'support.dart';

void main() {
  late FakeFlagSource source;
  late MemoryFlagCache cache;
  late FeatureFlags flags;

  setUp(() {
    source = FakeFlagSource();
    cache = MemoryFlagCache();
    flags = FeatureFlags(source: source, cache: cache);
  });

  group('mặc định an toàn', () {
    test('chưa load, chưa fetch: ra đúng mặc định của app', () {
      expect(flags.value(Flags.vixanhPayment), isTrue);
      expect(flags.value(Flags.newRouting), isFalse);
      expect(flags.value(Flags.quoteRefreshSeconds), 30);
    });

    test('load khi cache rỗng giữ mặc định', () async {
      await flags.load();
      expect(flags.value(Flags.vixanhPayment), isTrue);
      expect(flags.value(Flags.newRouting), isFalse);
    });

    test('cache đọc lỗi thì dùng mặc định, không ném', () async {
      cache
        ..stored = {'driver_new_routing_enabled': 'true'}
        ..failReads = true;
      await flags.load();
      expect(flags.value(Flags.newRouting), isFalse);
    });

    test('kill switch mặc định bật, tính năng mới mặc định tắt', () {
      expect(Flags.vixanhPayment.defaultValue, isTrue);
      expect(Flags.vixanhPayment.killSwitch, isTrue);
      expect(Flags.newRouting.defaultValue, isFalse);
    });
  });

  group('đọc giá trị từ server', () {
    Future<FeatureFlags> flagsWith(Map<String, Object?> values) async {
      final f = FeatureFlags(
        source: FakeFlagSource(values),
        cache: MemoryFlagCache(),
      );
      await f.refresh();
      await f.load(); // áp dụng cả cờ thường như ở lần mở app sau
      return f;
    }

    test('bool nhận bool và chuỗi, không phân biệt hoa thường', () async {
      for (final (raw, expected) in <(Object, bool)>[
        (true, true),
        ('true', true),
        ('TRUE', true),
        (' True ', true),
        (false, false),
        ('false', false),
      ]) {
        final f = await flagsWith({'driver_new_routing_enabled': raw});
        expect(f.value(Flags.newRouting), expected, reason: '$raw');
      }
    });

    test('bool từ chối "1", "yes", số: quay về mặc định', () async {
      // Khác SDK Firebase (asBool nhận "1"): ta chỉ nhận true/false.
      for (final raw in <Object?>['1', 'yes', 'on', 1, 0, '', null, 'banana']) {
        final f = await flagsWith({'driver_new_routing_enabled': raw});
        expect(f.value(Flags.newRouting), isFalse, reason: '$raw');
      }
    });

    test('int nhận số và chuỗi số trong khoảng', () async {
      expect(
        (await flagsWith({'quote_refresh_seconds': '45'}))
            .value(Flags.quoteRefreshSeconds),
        45,
      );
      expect(
        (await flagsWith({'quote_refresh_seconds': 60}))
            .value(Flags.quoteRefreshSeconds),
        60,
      );
    });

    test('int ngoài khoảng hoặc sai dạng: mặc định', () async {
      for (final raw in <Object?>[
        9,
        301,
        '5',
        '1000',
        '3.5',
        'abc',
        12.5,
        -1,
      ]) {
        final f = await flagsWith({'quote_refresh_seconds': raw});
        expect(f.value(Flags.quoteRefreshSeconds), 30, reason: '$raw');
      }
    });

    test('biên của khoảng: 10 và 300 hợp lệ', () async {
      expect(
        (await flagsWith({'quote_refresh_seconds': 10}))
            .value(Flags.quoteRefreshSeconds),
        10,
      );
      expect(
        (await flagsWith({'quote_refresh_seconds': 300}))
            .value(Flags.quoteRefreshSeconds),
        300,
      );
    });

    test('khoá lạ trong phản hồi bị bỏ qua và không vào cache', () async {
      source.values = {'khoa_la': 'true', 'driver_new_routing_enabled': 'true'};
      await flags.refresh();
      expect(cache.stored, {'driver_new_routing_enabled': true});
    });
  });

  group('kill switch có hiệu lực ngay, cờ thường chờ lần mở sau', () {
    test('refresh tắt VíXanh ngay trong phiên', () async {
      await flags.load();
      source.values = {'payment_vixanh_enabled': 'false'};

      expect(await flags.refresh(), isTrue);

      expect(flags.value(Flags.vixanhPayment), isFalse);
    });

    test('refresh không đổi cờ thường giữa phiên', () async {
      await flags.load();
      source.values = {
        'driver_new_routing_enabled': 'true',
        'quote_refresh_seconds': '60',
      };

      await flags.refresh();

      expect(flags.value(Flags.newRouting), isFalse);
      expect(flags.value(Flags.quoteRefreshSeconds), 30);
    });

    test('lần mở app sau (load) nhận cờ thường từ cache', () async {
      await flags.load();
      source.values = {'driver_new_routing_enabled': 'true'};
      await flags.refresh();

      final nextLaunch = FeatureFlags(source: FakeFlagSource(), cache: cache);
      await nextLaunch.load();

      expect(nextLaunch.value(Flags.newRouting), isTrue);
    });

    test('server gỡ khoá kill switch thì quay về mặc định ngay', () async {
      source.values = {'payment_vixanh_enabled': 'false'};
      await flags.refresh();
      expect(flags.value(Flags.vixanhPayment), isFalse);

      source.values = {};
      await flags.refresh();

      expect(flags.value(Flags.vixanhPayment), isTrue);
    });

    test('giá trị kill switch hỏng không làm tắt hay đổi gì', () async {
      source.values = {'payment_vixanh_enabled': 'tắt'};
      await flags.refresh();
      expect(flags.value(Flags.vixanhPayment), isTrue);
    });

    test(
      'kill switch trong cache từ phiên trước có hiệu lực khi load',
      () async {
        cache.stored = {'payment_vixanh_enabled': false};
        await flags.load();
        expect(flags.value(Flags.vixanhPayment), isFalse);
      },
    );
  });

  group('lỗi không lan ra ngoài', () {
    test('fetch lỗi: trả false, giữ nguyên giá trị đang dùng', () async {
      cache.stored = {'payment_vixanh_enabled': false};
      await flags.load();
      source.error = StateError('timeout');

      expect(await flags.refresh(), isFalse);

      expect(flags.value(Flags.vixanhPayment), isFalse);
      expect(cache.stored, {'payment_vixanh_enabled': false});
    });

    test('Error (bug trong lớp bọc) cũng được chặn', () async {
      source.error = AssertionError('bug');
      expect(await flags.refresh(), isFalse);
    });

    test(
      'ghi cache lỗi: refresh vẫn thành công và kill switch vẫn áp dụng',
      () async {
        cache.failWrites = true;
        source.values = {'payment_vixanh_enabled': 'false'};

        expect(await flags.refresh(), isTrue);
        expect(flags.value(Flags.vixanhPayment), isFalse);
      },
    );
  });

  test('value trả kiểu đúng với từng cờ', () {
    final Object a = flags.value(Flags.vixanhPayment);
    final Object b = flags.value(Flags.quoteRefreshSeconds);
    expect(a, isA<bool>());
    expect(b, isA<int>());
  });
}
