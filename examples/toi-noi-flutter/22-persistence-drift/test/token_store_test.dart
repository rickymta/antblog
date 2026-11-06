import 'package:flutter/services.dart'
    show MissingPluginException, PlatformException;
import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:flutter_secure_storage_platform_interface/flutter_secure_storage_platform_interface.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_22/auth/token_store.dart';

const tokens = AuthTokens(
  accessToken: 'test-access-1',
  refreshToken: 'test-refresh-1',
);

/// Kho Keychain/Keystore giả luôn lỗi, như khi Android không giải mã được.
class _BrokenPlatform extends FlutterSecureStoragePlatform {
  @override
  Future<void> write({
    required String key,
    required String value,
    required Map<String, String> options,
  }) => throw PlatformException(code: 'broken', message: 'secret-detail');

  @override
  Future<String?> read({
    required String key,
    required Map<String, String> options,
  }) => throw PlatformException(code: 'broken', message: 'secret-detail');

  @override
  Future<bool> containsKey({
    required String key,
    required Map<String, String> options,
  }) => throw PlatformException(code: 'broken');

  @override
  Future<void> delete({
    required String key,
    required Map<String, String> options,
  }) => throw PlatformException(code: 'broken');

  @override
  Future<Map<String, String>> readAll({required Map<String, String> options}) =>
      throw PlatformException(code: 'broken');

  @override
  Future<void> deleteAll({required Map<String, String> options}) =>
      throw PlatformException(code: 'broken');
}

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  // Cùng một bộ test chạy cho cả hai bản cài đặt của TokenStore.
  final factories = <String, TokenStore Function()>{
    'InMemoryTokenStore': InMemoryTokenStore.new,
    'SecureTokenStore': () {
      FlutterSecureStorage.setMockInitialValues({});
      return const SecureTokenStore(FlutterSecureStorage());
    },
  };

  for (final MapEntry(key: name, value: create) in factories.entries) {
    group('hợp đồng TokenStore: $name', () {
      late TokenStore store;
      setUp(() => store = create());

      test('chưa lưu thì đọc ra null', () async {
        expect(await store.read(), isNull);
      });

      test('lưu rồi đọc ra đúng cặp token', () async {
        await store.save(tokens);
        expect(await store.read(), tokens);
      });

      test('lưu lần hai thay thế cả hai token', () async {
        await store.save(tokens);
        const next = AuthTokens(
          accessToken: 'test-access-2',
          refreshToken: 'test-refresh-2',
        );
        await store.save(next);
        expect(await store.read(), next);
      });

      test('clear xoá token và gọi lại không lỗi', () async {
        await store.save(tokens);
        await store.clear();
        expect(await store.read(), isNull);
        await store.clear();
        expect(await store.read(), isNull);
      });
    });
  }

  group('SecureTokenStore', () {
    const store = SecureTokenStore(FlutterSecureStorage());

    test('cả hai token nằm trong một khoá duy nhất', () async {
      FlutterSecureStorage.setMockInitialValues({});
      await store.save(tokens);

      final all = await const FlutterSecureStorage().readAll();
      expect(all.keys, [SecureTokenStore.storageKey]);
    });

    test('JSON hỏng: trả null và xoá khoá', () async {
      FlutterSecureStorage.setMockInitialValues({
        SecureTokenStore.storageKey: '{not json',
      });

      expect(await store.read(), isNull);
      expect(
        await const FlutterSecureStorage().containsKey(
          key: SecureTokenStore.storageKey,
        ),
        isFalse,
      );
    });

    test('thiếu refresh hoặc sai kiểu: trả null và xoá khoá', () async {
      for (final raw in [
        '{"access":"a"}',
        '{"access":"a","refresh":1}',
        '[]',
      ]) {
        FlutterSecureStorage.setMockInitialValues({
          SecureTokenStore.storageKey: raw,
        });
        expect(await store.read(), isNull, reason: raw);
        expect(
          await const FlutterSecureStorage().containsKey(
            key: SecureTokenStore.storageKey,
          ),
          isFalse,
          reason: raw,
        );
      }
    });

    test('token chứa ký tự đặc biệt đi qua JSON nguyên vẹn', () async {
      FlutterSecureStorage.setMockInitialValues({});
      const odd = AuthTokens(
        accessToken: 'a"b\\c\n"d',
        refreshToken: 'tiếng Việt 😀',
      );
      await store.save(odd);
      expect(await store.read(), odd);
    });
  });

  group('khi kho của hệ điều hành lỗi', () {
    late FlutterSecureStoragePlatform original;
    setUp(() {
      original = FlutterSecureStoragePlatform.instance;
      FlutterSecureStoragePlatform.instance = _BrokenPlatform();
    });
    tearDown(() => FlutterSecureStoragePlatform.instance = original);

    const store = SecureTokenStore(FlutterSecureStorage());

    test('read coi như chưa đăng nhập', () async {
      expect(await store.read(), isNull);
    });

    test('clear (đăng xuất) không ném lỗi', () async {
      await store.clear();
    });

    test('save ném lỗi để màn hình báo cho người dùng', () async {
      await expectLater(store.save(tokens), throwsA(isA<PlatformException>()));
    });
  });

  test('không mock thì FlutterSecureStorage gọi plugin và ném MissingPluginException', () async {
    // Chỉ để thấy vì sao test phải mock: trên máy test không có plugin.
    final original = FlutterSecureStoragePlatform.instance;
    addTearDown(() => FlutterSecureStoragePlatform.instance = original);
    FlutterSecureStoragePlatform.instance = MethodChannelFlutterSecureStorage();
    await expectLater(
      const FlutterSecureStorage().read(key: 'k'),
      throwsA(isA<MissingPluginException>()),
    );
  });

  group('AuthTokens', () {
    test('toString không lộ giá trị token', () {
      expect(tokens.toString(), isNot(contains('test-access-1')));
      expect(tokens.toString(), isNot(contains('test-refresh-1')));
    });

    test('so sánh theo giá trị', () {
      expect(
        tokens,
        const AuthTokens(
          accessToken: 'test-access-1',
          refreshToken: 'test-refresh-1',
        ),
      );
      expect(tokens.hashCode, tokens.hashCode);
    });
  });
}
