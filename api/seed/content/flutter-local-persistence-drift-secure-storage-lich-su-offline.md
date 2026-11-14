---
slug: flutter-local-persistence-drift-secure-storage-lich-su-offline
title: "Lưu trữ cục bộ: drift, shared_preferences và secure storage"
summary: "Bài 22: chọn kho lưu trên máy theo loại dữ liệu của Tới Nơi. drift (SQLite) cho lịch sử chuyến với bảng, DAO, stream query và migration có test; SharedPreferencesAsync cho chế độ sáng/tối; flutter_secure_storage sau interface TokenStore cho token. Test chạy không cần thiết bị."
category: flutter
level: intermediate
tags: [flutter, dart, mobile, toi-noi, drift, sqlite, shared-preferences, secure-storage, persistence]
language: vi
---

Khách của Tới Nơi vào hầm gửi xe, mất sóng, mở app để xem hôm qua mình trả bao nhiêu cho chuyến về nhà. Màn hình trống trơn, vì lịch sử chuyến chỉ sống trong bộ nhớ và mỗi lần mở app là tải lại từ server. Cùng lúc app quên chế độ tối khách đã chọn, và bắt đăng nhập lại vì token cũng không được giữ.

Đó là ba loại dữ liệu khác nhau, và cám dỗ là nhét hết vào một kho. Bài này tách chúng ra: lịch sử chuyến vào SQLite qua **drift**, chế độ sáng/tối vào **shared_preferences**, token vào kho của hệ điều hành qua **flutter_secure_storage** đứng sau interface `TokenStore`.

Đây là bài 22 của lộ trình **Flutter qua Tới Nơi**. Ta nên đọc [bài 21 về Clean Architecture](/articles/flutter-clean-architecture-domain-data-presentation-mobile) trước, vì `TripHistoryRepository` đứng đúng ranh giới domain và data như bài đó. Provider nối kho với màn hình là của [bài 18 về Riverpod](/articles/flutter-riverpod-provider-notifier-asyncnotifier-dat-chuyen), còn `Trip` và `Money` là của package ở [bài 06](/articles/dart-package-pubspec-lint-test-toi-noi-core). Ví dụ lấy từ hồ sơ [Tới Nơi](/articles/du-an-mau-toi-noi), bảng giá là số minh hoạ. OTP, refresh và interceptor sẽ nằm ở [bài 24](/articles/flutter-authentication-otp-jwt-refresh-token-secure-storage).

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Màn hình lịch sử đang <code>watch</code> bảng <code>trips</code> của drift. Một đoạn code chèn thêm chuyến bằng <code>customStatement('INSERT INTO trips ...')</code>. Danh sách trên màn hình có tự cập nhật không?</p>
<details><summary>Xem đáp án</summary><p>Không. <code>customStatement</code> chạy SQL nhưng không cho drift biết bảng nào vừa đổi, nên stream query đứng yên (test <code>trip_dao_test.dart</code> kiểm đúng điều này). Gọi thêm <code>notifyUpdates</code>, hoặc dùng <code>customInsert(..., updates: {...})</code>, thì stream mới phát lại. Mục "DAO, repository và stream query" giải thích cơ chế.</p></details>
</div>

## Chọn kho theo loại dữ liệu

Trước khi chọn thư viện, ta hỏi: dữ liệu có là bí mật không, có nhiều dòng cần truy vấn không, mất đi thì sao?

| Dữ liệu của Tới Nơi | Đặc điểm | Kho | Điều cần nhớ |
|---|---|---|---|
| Lịch sử chuyến | Nhiều dòng, cần sắp xếp và giới hạn, tải lại được | `drift` (SQLite) | Schema có phiên bản, đổi schema phải có migration |
| Chế độ sáng/tối | Một giá trị nhỏ, mất đi chỉ phải chọn lại | `shared_preferences` | README của gói: không đảm bảo ghi xuống đĩa ngay, không dùng cho dữ liệu quan trọng |
| Access/refresh token | Bí mật, nhỏ | `flutter_secure_storage` | Keychain trên iOS/macOS; Android mã hoá bằng khoá giữ trong Keystore |

Hình 1 cho thấy ba kho nằm sau ba interface. Màn hình chỉ biết `TripHistoryRepository`, `AppPreferences` và `TokenStore`, nên đổi kho hay thay bản giả trong test chỉ cần đổi một provider.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 370" role="img" aria-label="Hình 1. Màn hình và provider gọi ba interface. TripHistoryRepository dùng drift và file SQLite toi_noi.sqlite. AppPreferences dùng SharedPreferencesAsync, lưu ở NSUserDefaults trên iOS hoặc DataStore trên Android. TokenStore dùng SecureTokenStore với flutter_secure_storage, lưu ở Keychain trên iOS; trên Android là tệp mã hoá, còn Keystore giữ khoá RSA bọc khoá mã hoá."><defs><marker id="fl22-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker><marker id="fl22-1-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs>
<rect x="10" y="100" width="140" height="170" rx="10" class="f-phys-soft s-phys"/><text x="24" y="150" class="t-b">Màn hình</text><text x="24" y="176" class="t-b">và provider</text><text x="24" y="206" class="t-small">Riverpod</text>
<rect x="190" y="20" width="210" height="90" rx="10" class="f-pk-soft s-pk"/><text x="204" y="52" class="t-mono">TripHistoryRepository</text><text x="204" y="80" class="t-small">lịch sử chuyến</text>
<rect x="440" y="20" width="210" height="90" rx="10" class="f-pk-soft s-pk"/><text x="454" y="52" class="t-b">drift (SQLite)</text><text x="454" y="80" class="t-small">TripDao, stream query</text>
<rect x="690" y="20" width="180" height="90" rx="10" class="f-phys-soft s-phys"/><text x="704" y="52" class="t-mono">toi_noi.sqlite</text><text x="704" y="80" class="t-small">file nội bộ của app</text>
<rect x="190" y="140" width="210" height="90" rx="10" class="f-pk-soft s-pk"/><text x="204" y="172" class="t-mono">AppPreferences</text><text x="204" y="200" class="t-small">chế độ sáng/tối</text>
<rect x="440" y="140" width="210" height="90" rx="10" class="f-pk-soft s-pk"/><text x="454" y="172" class="t-mono">SharedPreferencesAsync</text><text x="454" y="200" class="t-small">không cache riêng</text>
<rect x="690" y="140" width="180" height="90" rx="10" class="f-phys-soft s-phys"/><text x="704" y="168" class="t-b">iOS: NSUserDefaults</text><text x="704" y="196" class="t-small">Android: DataStore</text>
<rect x="190" y="260" width="210" height="90" rx="10" class="f-pk-soft s-pk"/><text x="204" y="292" class="t-mono">TokenStore</text><text x="204" y="320" class="t-small">token đăng nhập</text>
<rect x="440" y="260" width="210" height="90" rx="10" class="f-pk-soft s-pk"/><text x="454" y="292" class="t-b">SecureTokenStore</text><text x="454" y="320" class="t-small">flutter_secure_storage</text>
<rect x="690" y="260" width="180" height="90" rx="10" class="f-warn-soft s-warn"/><text x="704" y="284" class="t-b">iOS: Keychain</text><text x="704" y="308" class="t-small">Android: tệp mã hoá</text><text x="704" y="328" class="t-small">khoá RSA ở Keystore</text>
<path d="M152 160 L188 70" class="f-none s-pk" stroke-width="2" marker-end="url(#fl22-1-a)"/>
<path d="M152 185 L188 185" class="f-none s-pk" stroke-width="2" marker-end="url(#fl22-1-a)"/>
<path d="M152 210 L188 300" class="f-none s-pk" stroke-width="2" marker-end="url(#fl22-1-a)"/>
<path d="M402 65 L438 65" class="f-none s-pk" stroke-width="2" marker-end="url(#fl22-1-a)"/>
<path d="M652 65 L688 65" class="f-none s-pk" stroke-width="2" marker-end="url(#fl22-1-a)"/>
<path d="M402 185 L438 185" class="f-none s-pk" stroke-width="2" marker-end="url(#fl22-1-a)"/>
<path d="M652 185 L688 185" class="f-none s-pk" stroke-width="2" marker-end="url(#fl22-1-a)"/>
<path d="M402 305 L438 305" class="f-none s-pk" stroke-width="2" marker-end="url(#fl22-1-a)"/>
<path d="M652 305 L688 305" class="f-none s-warn" stroke-width="2" marker-end="url(#fl22-1-w)"/>
</svg>
</div>
<figcaption>Hình 1. Ba loại dữ liệu đi qua ba interface tới ba kho. Mũi tên là hướng gọi. Hàng token có màu cảnh báo vì đó là bí mật.</figcaption>
</figure>

Dưới đây là các gói của lab (hai gói `platform_interface` chỉ dùng để thay nền tảng trong test) và chỗ nối ba kho thành provider:

### pubspec.yaml · 22-persistence-drift/pubspec.yaml (trích)

```yaml
dependencies:
  drift: ^2.35.3
  drift_flutter: ^0.3.1
  flutter:
    sdk: flutter
  flutter_riverpod: ^3.5.0
  flutter_secure_storage: ^11.2.0
  path_provider: ^2.1.6
  shared_preferences: ^2.5.6
  toi_noi_core:
    path: ../packages/toi_noi_core
  toi_noi_ui:
    path: ../packages/toi_noi_ui

dev_dependencies:
  build_runner: ^2.17.1
  drift_dev: ^2.35.2
  flutter_lints: ^6.0.0
  flutter_secure_storage_platform_interface: ^2.1.1
  flutter_test:
    sdk: flutter
  shared_preferences_platform_interface: ^2.4.2

flutter:
  uses-material-design: true
```

### providers.dart · 22-persistence-drift/lib/providers.dart (trích)

```dart
/// Một `AppDatabase` cho cả app; đóng khi `ProviderScope` bị huỷ.
final appDatabaseProvider = Provider<AppDatabase>((ref) {
  final database = AppDatabase();
  ref.onDispose(database.close);
  return database;
});

final tripHistoryRepositoryProvider = Provider<TripHistoryRepository>(
  (ref) => DriftTripHistoryRepository(ref.watch(appDatabaseProvider).tripDao),
);

final appPreferencesProvider = Provider<AppPreferences>(
  (ref) => AppPreferences(SharedPreferencesAsync()),
);

final tokenStoreProvider = Provider<TokenStore>(
  (ref) => const SecureTokenStore(FlutterSecureStorage()),
);
```

## drift: bảng, kiểu dữ liệu và mã sinh

drift mô tả bảng SQLite bằng class Dart, rồi `drift_dev` sinh phần truy vấn có kiểu. Bảng `Trips` bám quy ước của loạt bài: cước là `int` đồng, quãng đường là `int` mét, thời điểm là UTC.

### trips_table.dart · 22-persistence-drift/lib/db/trips_table.dart

```dart
import 'package:drift/drift.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

/// Bảng lịch sử chuyến đi lưu trên máy (schema phiên bản 2).
@DataClassName('TripRow')
class Trips extends Table {
  /// Mã chuyến do server cấp, dùng làm khoá chính.
  TextColumn get id => text()();

  /// Tên của enum `TripStatus` (ví dụ `completed`), không lưu chỉ số.
  TextColumn get status => textEnum<TripStatus>()();

  /// Quãng đường bằng mét nguyên.
  IntColumn get distanceMeters => integer()();

  /// Cước bằng đồng nguyên; database từ chối số âm.
  // ignore: recursive_getters
  IntColumn get fareVnd => integer().check(fareVnd.isBiggerOrEqualValue(0))();

  /// Thời điểm đặt, UTC, tính bằng mili giây từ epoch.
  IntColumn get requestedAtMs => integer()();

  /// Tên của enum `ServiceKind`. Thêm ở schema 2; dòng cũ nhận `ride`.
  TextColumn get serviceKind =>
      textEnum<ServiceKind>().withDefault(Constant(ServiceKind.ride.name))();

  /// Tài xế nhận chuyến; null khi chưa có. Thêm ở schema 2.
  TextColumn get driverId => text().nullable()();

  @override
  Set<Column<Object>> get primaryKey => {id};
}
```

- **Thời điểm lưu bằng mili giây UTC** trong cột `int`: ta tự quyết định định dạng trên đĩa, múi giờ là việc của giao diện.
- **`textEnum` lưu tên enum** (`completed`), không lưu chỉ số, nên đổi thứ tự giá trị không làm dòng cũ đổi nghĩa. Xoá hay đổi tên một giá trị thì dòng cũ mang tên lạ và lần đọc sẽ ném `ArgumentError`; test chèn tên `teleporting` để chứng minh.
- **`check` đặt luật trong database**: cước âm bị SQLite từ chối bằng `CHECK constraint failed: fare_vnd`, và lệnh ghi nhiều dòng chạy trong một transaction nên cả lô không được ghi.

Hai cột cuối thuộc schema 2. Giờ đến class database:

### app_database.dart · 22-persistence-drift/lib/db/app_database.dart (trích)

```dart
import 'package:drift/drift.dart';
import 'package:drift_flutter/drift_flutter.dart';
import 'package:path_provider/path_provider.dart';
// Mã sinh ra trong app_database.g.dart dùng TripStatus và ServiceKind từ đây.
import 'package:toi_noi_core/toi_noi_core.dart';

import 'app_database.steps.dart';
import 'trip_dao.dart';
import 'trips_table.dart';

part 'app_database.g.dart';

/// Database SQLite của app: hiện chỉ có lịch sử chuyến.
///
/// Constructor nhận `QueryExecutor` tuỳ chọn: app để trống và dùng file thật
/// qua `drift_flutter`, test truyền `NativeDatabase.memory()` hoặc một kết nối
/// tới file do `SchemaVerifier` dựng.
@DriftDatabase(tables: [Trips], daos: [TripDao])
class AppDatabase extends _$AppDatabase {
  AppDatabase([QueryExecutor? executor]) : super(executor ?? _openOnDevice());

  /// File `toi_noi.sqlite` trong thư mục application support: chỗ dành cho
  /// file nội bộ của app, không phải thư mục dữ liệu người dùng. drift_flutter
  /// mở nó trong một isolate nền nên truy vấn không chặn UI.
  static QueryExecutor _openOnDevice() => driftDatabase(
    name: 'toi_noi',
    native: const DriftNativeOptions(
      databaseDirectory: getApplicationSupportDirectory,
    ),
  );

  /// Tăng mỗi khi schema đổi, và thêm một bước `fromXToY` tương ứng.
  @override
  int get schemaVersion => 2;
  // ... migration: mục "Migration từ schema 1 lên 2"
}
```

Constructor nhận `QueryExecutor` tuỳ chọn: app để trống và mở file thật, test truyền kết nối trong bộ nhớ. Trên thiết bị, `drift_flutter` mở file trong một isolate nền (`NativeDatabase.createBackgroundConnection`) nên truy vấn không chặn UI. Ta đặt file trong thư mục application support, nơi `path_provider` nói dành cho tệp không lộ ra người dùng; thư mục documents mặc định của `drift_flutter` dành cho dữ liệu người dùng tạo ra.

Sinh mã bằng `dart run build_runner build`. Lab **commit cả các file sinh** (`*.g.dart`, `app_database.steps.dart`, thư mục test sinh) nên `flutter test` xanh ngay sau `flutter pub get`; `.gitignore` không loại `*.g.dart`.

SQLite của test đến từ đâu? `drift_flutter` kéo theo `sqlite3` 3.x, dùng build hook của Dart: khi `.dart_tool` chưa có bản tải sẵn (mỗi job CI sạch đều vậy), hook tải thư viện SQLite biên dịch sẵn từ GitHub Releases của gói, có kiểm sha256. Vậy không cần cài `libsqlite3` trên máy hay CI ubuntu, nhưng cần mạng tới github.com. Ta kiểm trên macOS; bản `x64.linux` có trong danh sách tệp của gói nhưng chưa chạy ở lab này.

## DAO, repository và stream query

DAO gom mọi truy vấn trên bảng và trả về `TripRow`, không biết `Trip` của domain:

### trip_dao.dart · 22-persistence-drift/lib/db/trip_dao.dart (trích)

```dart
  /// Chèn hoặc ghi đè theo `id`: cùng một chuyến tải lại từ server không nhân đôi.
  Future<void> upsertAll(List<TripsCompanion> rows) =>
      batch((batch) => batch.insertAllOnConflictUpdate(trips, rows));

  /// Stream [limit] chuyến gần nhất, mới trước. Phát lại mỗi khi bảng đổi.
  Stream<List<TripRow>> watchRecent({int limit = 20}) {
    final query = select(trips)
      ..orderBy([
        (t) => OrderingTerm.desc(t.requestedAtMs),
        // Cùng thời điểm thì xếp theo id để thứ tự luôn cố định.
        (t) => OrderingTerm.asc(t.id),
      ])
      ..limit(limit);
    return query.watch();
  }
```

`watchRecent` trả về `Stream`: mỗi lần bảng `trips` đổi, drift chạy lại câu `SELECT` và phát danh sách mới (Hình 2).

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 340" role="img" aria-label="Hình 2. Lệnh ghi qua API của drift làm bảng trips đổi, drift ghi nhận bảng đã đổi, chạy lại câu SELECT của watchRecent, rồi StreamProvider phát AsyncData mới cho màn hình. customStatement làm SQLite đổi nhưng drift không biết, nên stream đứng yên. notifyUpdates báo tay cho drift."><defs><marker id="fl22-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker><marker id="fl22-2-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs>
<rect x="8" y="30" width="144" height="80" rx="10" class="f-pk-soft s-pk"/><text x="20" y="62" class="t-b">Ghi qua drift</text><text x="20" y="88" class="t-small">upsertAll</text>
<rect x="188" y="30" width="144" height="80" rx="10" class="f-phys-soft s-phys"/><text x="200" y="62" class="t-b">SQLite đổi</text><text x="200" y="88" class="t-small">bảng trips</text>
<rect x="368" y="30" width="144" height="80" rx="10" class="f-pk-soft s-pk"/><text x="380" y="62" class="t-b">drift ghi nhận</text><text x="380" y="88" class="t-small">bảng trips đổi</text>
<rect x="548" y="30" width="144" height="80" rx="10" class="f-pk-soft s-pk"/><text x="560" y="62" class="t-b">Chạy lại SELECT</text><text x="560" y="88" class="t-small">của watchRecent</text>
<rect x="728" y="30" width="144" height="80" rx="10" class="f-phys-soft s-phys"/><text x="740" y="62" class="t-b">Provider phát</text><text x="740" y="88" class="t-small">AsyncData mới</text>
<path d="M154 70 L186 70" class="f-none s-pk" stroke-width="2" marker-end="url(#fl22-2-a)"/>
<path d="M334 70 L366 70" class="f-none s-pk" stroke-width="2" marker-end="url(#fl22-2-a)"/>
<path d="M514 70 L546 70" class="f-none s-pk" stroke-width="2" marker-end="url(#fl22-2-a)"/>
<path d="M694 70 L726 70" class="f-none s-pk" stroke-width="2" marker-end="url(#fl22-2-a)"/>
<rect x="8" y="200" width="160" height="80" rx="10" class="f-warn-soft s-warn"/><text x="20" y="232" class="t-mono">customStatement</text><text x="20" y="258" class="t-small">INSERT thô</text>
<path d="M90 198 L225 112" class="f-none s-warn" stroke-width="2" stroke-dasharray="8 6" marker-end="url(#fl22-2-w)"/>
<text x="20" y="168" class="t-small t-warn">drift không biết gì</text>
<rect x="368" y="200" width="144" height="80" rx="10" class="f-pk-soft s-pk"/><text x="380" y="232" class="t-mono">notifyUpdates</text><text x="380" y="258" class="t-small">báo tay bảng</text>
<path d="M440 198 L440 112" class="f-none s-pk" stroke-width="2" marker-end="url(#fl22-2-a)"/>
<text x="8" y="318" class="t-small">SQL thô làm đổi dữ liệu nhưng không qua bước "drift ghi nhận": stream đứng yên cho tới khi có báo tay.</text>
</svg>
</div>
<figcaption>Hình 2. Đường từ lệnh ghi tới màn hình. <code>customStatement</code> làm SQLite đổi nhưng drift không biết, nên phải báo tay bằng <code>notifyUpdates</code>.</figcaption>
</figure>

Hai hành vi này được đo bằng test (`row` là hàm dựng `TripsCompanion` ở đầu file test):

### trip_dao_test.dart · 22-persistence-drift/test/db/trip_dao_test.dart (trích)

```dart
    test('ghi lại dữ liệu y hệt vẫn phát lại', () async {
      final emitted = <int>[];
      final sub = db.tripDao.watchRecent().listen(
        (rows) => emitted.add(rows.length),
      );
      addTearDown(sub.cancel);

      await pumpEventQueue();
      await db.tripDao.upsertAll([row('T1')]);
      await pumpEventQueue();
      await db.tripDao.upsertAll([row('T1')]);
      await pumpEventQueue();

      expect(emitted, [0, 1, 1]);
    });

    test('customStatement không báo stream, notifyUpdates thì có', () async {
      final emitted = <int>[];
      final sub = db.tripDao.watchRecent().listen(
        (rows) => emitted.add(rows.length),
      );
      addTearDown(sub.cancel);
      await pumpEventQueue();

      await db.customStatement(
        "INSERT INTO trips (id, status, distance_meters, fare_vnd, "
        "requested_at_ms) VALUES ('T1', 'completed', 5000, 25500, 1)",
      );
      await pumpEventQueue();
      expect(emitted, [0], reason: 'stream chưa biết bảng đã đổi');

      db.notifyUpdates({
        TableUpdate.onTable(db.trips, kind: UpdateKind.insert),
      });
      await pumpEventQueue();
      expect(emitted, [0, 1]);
    });
```

Ghi lại đúng dữ liệu cũ vẫn làm stream phát lại, vì drift không so kết quả mới với cũ. Còn `customStatement` không báo gì, vì drift không đọc hiểu SQL thô; trang "Stream queries" của drift mô tả cùng hướng: ghi ngoài API của nó không kích hoạt cập nhật, và ta báo tay bằng `notifyUpdates`.

Repository là ranh giới giữa dòng database và domain:

### trip_history_repository.dart · 22-persistence-drift/lib/trip_history_repository.dart (trích)

```dart
extension TripRowToTrip on TripRow {
  Trip toTrip() => Trip(
    id: id,
    serviceKind: serviceKind,
    status: status,
    distanceKm: distanceMeters / 1000,
    fare: Money.fromVnd(fareVnd),
    driverId: driverId,
    requestedAt: DateTime.fromMillisecondsSinceEpoch(
      requestedAtMs,
      isUtc: true,
    ),
  );
}

extension TripToCompanion on Trip {
  TripsCompanion toCompanion() => TripsCompanion(
    id: Value(id),
    serviceKind: Value(serviceKind),
    status: Value(status),
    distanceMeters: Value((distanceKm * 1000).round()),
    fareVnd: Value(fare.vnd),
    driverId: Value(driverId),
    requestedAtMs: Value(requestedAt.millisecondsSinceEpoch),
  );
}
```

Dữ liệu đọc từ đĩa là dữ liệu ngoài: `Money.fromVnd` và constructor của `Trip` kiểm lại, nên dòng `accepted` không có `driverId` ném `ArgumentError` thay vì trở thành chuyến sai. Chiều ghi đổi quãng đường sang mét nguyên và cắt thời điểm xuống mili giây; test kiểm cả hai, cùng các trường hợp 2,001 km (12.005 VND) và 100 km (453.000 VND). Một dòng hỏng làm cả stream báo lỗi, nên màn hình xử lý nhánh lỗi:

### history_screen.dart · 22-persistence-drift/lib/history_screen.dart (trích)

```dart
      body: switch (trips) {
        // Lỗi sau một lần có dữ liệu vẫn giữ `.value` cũ; đừng dùng `.value`
        // để quyết định hiện danh sách, hãy khớp theo AsyncError/AsyncData.
        AsyncError() => _ErrorView(
          onClear: () => ref.read(tripHistoryRepositoryProvider).clear(),
        ),
        AsyncData(:final value) when value.isEmpty => const Center(
          child: Text('Chưa có chuyến nào'),
        ),
        AsyncData(:final value) => ListView.separated(
          itemCount: value.length,
          separatorBuilder: (_, _) => const Divider(height: 1),
          itemBuilder: (_, index) => _TripTile(trip: value[index]),
        ),
        _ => const Center(child: CircularProgressIndicator()),
      },
```

Lịch sử chỉ là bản sao của dữ liệu trên server, nên lối ra là nút "Xoá bộ nhớ đệm" rồi tải lại. Dữ liệu không có bản gốc nào khác thì không làm vậy được; bài outbox ở Phần 6 sẽ quay lại.

## Migration từ schema 1 lên 2

drift ghi phiên bản schema vào `PRAGMA user_version` của file, và mỗi lần mở so nó với `schemaVersion` trong code. Giả sử schema 1 chưa lưu loại dịch vụ và tài xế, schema 2 thêm hai cột đó. Bốn bước với công cụ của drift:

1. Khai báo database trong `build.yaml`:

   ```yaml
   targets:
     $default:
       builders:
         drift_dev:
           options:
             databases:
               app_database: lib/db/app_database.dart
             # Giữ mặc định: ảnh chụp schema ở drift_schemas/, test sinh ra ở test/drift/
   ```

2. Khi còn ở schema 1, chạy `dart run drift_dev make-migrations` để có `drift_schemas/app_database/drift_schema_v1.json`.
3. Sửa bảng, tăng `schemaVersion` lên 2, chạy lại lệnh đó: ta có thêm `drift_schema_v2.json`, `app_database.steps.dart` với `stepByStep`, và test migration trong `test/drift/app_database/`.
4. Viết bước `from1To2`:

### app_database.dart · 22-persistence-drift/lib/db/app_database.dart (trích)

```dart
  @override
  MigrationStrategy get migration => MigrationStrategy(
    onUpgrade: stepByStep(
      // Schema 2: thêm loại dịch vụ và tài xế. Dòng cũ nhận `ride` nhờ giá trị
      // mặc định của cột; `addColumn` chạy ALTER TABLE ... ADD COLUMN.
      from1To2: (m, schema) async {
        await m.addColumn(schema.trips, schema.trips.serviceKind);
        await m.addColumn(schema.trips, schema.trips.driverId);
      },
    ),
  );
```

`addColumn` chạy `ALTER TABLE ... ADD COLUMN` và SQLite đặt cột mới ở **cuối** bảng, nên `serviceKind` và `driverId` được khai báo ở cuối `Trips` để bảng tạo mới và bảng đã nâng cấp có cùng thứ tự cột. `serviceKind` là `NOT NULL` nên có mặc định `ride` cho dòng cũ.

Hình 3 vẽ ba trường hợp khi mở file; trường hợp thứ ba là người dùng cài lại bản app cũ.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 3. Ba trường hợp khi mở file database với schemaVersion của code là 2. File ở version 1 nhỏ hơn 2 nên gọi onUpgrade và chạy from1To2. File ở version 2 bằng 2 nên mở bình thường. File ở version 3 lớn hơn 2 vì người dùng hạ phiên bản app, nên stepByStep ném StateError."><defs><marker id="fl22-3-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker><marker id="fl22-3-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs>
<rect x="10" y="20" width="220" height="70" rx="10" class="f-phys-soft s-phys"/><text x="24" y="50" class="t-b">File ở version 1</text><text x="24" y="74" class="t-small">user_version = 1</text>
<rect x="300" y="20" width="270" height="70" rx="10" class="f-pk-soft s-pk"/><text x="314" y="50" class="t-b">1 &lt; 2: gọi onUpgrade</text><text x="314" y="74" class="t-small">schemaVersion của code là 2</text>
<rect x="640" y="20" width="230" height="70" rx="10" class="f-pk-soft s-pk"/><text x="654" y="50" class="t-b">Chạy from1To2</text><text x="654" y="74" class="t-small">cột mới, dòng cũ = ride</text>
<path d="M232 55 L298 55" class="f-none s-pk" stroke-width="2" marker-end="url(#fl22-3-a)"/>
<path d="M572 55 L638 55" class="f-none s-pk" stroke-width="2" marker-end="url(#fl22-3-a)"/>
<rect x="10" y="115" width="220" height="70" rx="10" class="f-phys-soft s-phys"/><text x="24" y="145" class="t-b">File ở version 2</text><text x="24" y="169" class="t-small">user_version = 2</text>
<rect x="300" y="115" width="270" height="70" rx="10" class="f-pk-soft s-pk"/><text x="314" y="145" class="t-b">2 = 2: không migration</text><text x="314" y="169" class="t-small">schemaVersion của code là 2</text>
<rect x="640" y="115" width="230" height="70" rx="10" class="f-pk-soft s-pk"/><text x="654" y="145" class="t-b">Mở bình thường</text><text x="654" y="169" class="t-small">không chạy bước nào</text>
<path d="M232 150 L298 150" class="f-none s-pk" stroke-width="2" marker-end="url(#fl22-3-a)"/>
<path d="M572 150 L638 150" class="f-none s-pk" stroke-width="2" marker-end="url(#fl22-3-a)"/>
<rect x="10" y="210" width="220" height="70" rx="10" class="f-warn-soft s-warn"/><text x="24" y="240" class="t-b">File ở version 3</text><text x="24" y="264" class="t-small">user_version = 3</text>
<rect x="300" y="210" width="270" height="70" rx="10" class="f-warn-soft s-warn"/><text x="314" y="240" class="t-b">3 &gt; 2: người dùng hạ phiên bản app</text><text x="314" y="264" class="t-small">schemaVersion của code là 2</text>
<rect x="640" y="210" width="230" height="70" rx="10" class="f-warn-soft s-warn"/><text x="654" y="240" class="t-b">StateError</text><text x="654" y="264" class="t-small">các bước chỉ đi lên</text>
<path d="M232 245 L298 245" class="f-none s-warn" stroke-width="2" marker-end="url(#fl22-3-w)"/>
<path d="M572 245 L638 245" class="f-none s-warn" stroke-width="2" marker-end="url(#fl22-3-w)"/>
</svg>
</div>
<figcaption>Hình 3. drift so <code>user_version</code> của file với <code>schemaVersion</code> của code: nhỏ hơn thì chạy các bước, bằng nhau thì mở luôn, lớn hơn thì <code>stepByStep</code> ném lỗi vì các bước chỉ đi lên.</figcaption>
</figure>

Test do drift sinh dựng database ở từng phiên bản cũ, chạy migration rồi so schema thật với schema kỳ vọng. Ta bổ sung một test kiểm dữ liệu:

### migration_test.dart · 22-persistence-drift/test/drift/app_database/migration_test.dart (trích)

```dart
  test('migration from v1 to v2 does not corrupt data', () async {
    const oldTripsData = <v1.TripsData>[
      v1.TripsData(
        id: 'T1',
        status: 'completed',
        distanceMeters: 5000,
        fareVnd: 25500,
        requestedAtMs: 1762158600000,
      ),
    ];
    const expectedNewTripsData = <v2.TripsData>[
      v2.TripsData(
        id: 'T1',
        status: 'completed',
        distanceMeters: 5000,
        fareVnd: 25500,
        requestedAtMs: 1762158600000,
        serviceKind: 'ride',
      ),
    ];

    await verifier.testWithDataIntegrity(
      oldVersion: 1,
      newVersion: 2,
      createOld: v1.DatabaseAtV1.new,
      createNew: v2.DatabaseAtV2.new,
      openTestedDatabase: AppDatabase.new,
      createItems: (batch, oldDb) {
        batch.insertAll(oldDb.trips, oldTripsData);
      },
      validateItems: (newDb) async {
        expect(await newDb.select(newDb.trips).get(), expectedNewTripsData);
      },
    );
  });
```

Để chắc test bắt được lỗi, ta xoá thử dòng `addColumn` của `driverId`: cả hai test đỏ với `Schema does not match`, `comparing driver_id`, `The actual schema does not contain anything with this name`. Rồi ta trả dòng đó về chỗ cũ. Test thứ ba trong `migration_from_old_file_test.dart` không cần file sinh: nó dựng tay database schema 1 bằng `NativeDatabase.memory(setup: ...)`, mở bằng code hiện tại và kiểm `user_version` thành 2, dòng cũ nhận `ride`. Cùng file còn ghi lại trường hợp hạ phiên bản: `StateError` nói các bước chỉ nâng được. Lab chưa xử lý hạ phiên bản; với bảng chỉ là bản sao, xoá và dựng lại là hợp lý, còn dữ liệu duy nhất thì không.

## shared_preferences cho tuỳ chọn nhỏ

`shared_preferences` có ba API: `SharedPreferences` là API cũ mà README nói sẽ bị deprecated, `SharedPreferencesWithCache` giữ cache để đọc đồng bộ, `SharedPreferencesAsync` không cache và luôn đọc mới từ nền tảng. README cảnh báo cache có thể sai khi dùng nhiều isolate hoặc engine (ví dụ plugin tạo ngữ cảnh nền như `firebase_messaging`), mà Tới Nơi về sau sẽ có cả hai. Ta chọn bản không cache:

### app_preferences.dart · 22-persistence-drift/lib/preferences/app_preferences.dart

```dart
import 'package:flutter/material.dart' show ThemeMode;
import 'package:shared_preferences/shared_preferences.dart';

/// Tuỳ chọn nhỏ của người dùng, lưu bằng `SharedPreferencesAsync`.
///
/// Chỉ để dữ liệu mất đi cũng không sao: nếu khoá thiếu hoặc giá trị lạ, app
/// quay về mặc định thay vì lỗi.
class AppPreferences {
  AppPreferences(this._prefs);

  final SharedPreferencesAsync _prefs;

  static const themeModeKey = 'settings.theme_mode';

  /// Chế độ sáng/tối đã chọn; `ThemeMode.system` khi chưa chọn hoặc giá trị lạ.
  Future<ThemeMode> readThemeMode() async {
    final name = await _prefs.getString(themeModeKey);
    return ThemeMode.values.asNameMap()[name] ?? ThemeMode.system;
  }

  Future<void> writeThemeMode(ThemeMode mode) =>
      _prefs.setString(themeModeKey, mode.name);
}
```

Giá trị lưu là tên `ThemeMode`; thiếu khoá hoặc giá trị lạ thì về `ThemeMode.system`. Notifier kiểm `ref.mounted` sau khi chờ ghi:

### providers.dart · 22-persistence-drift/lib/providers.dart (trích)

```dart
class ThemeModeNotifier extends AsyncNotifier<ThemeMode> {
  @override
  Future<ThemeMode> build() =>
      ref.watch(appPreferencesProvider).readThemeMode();

  Future<void> select(ThemeMode mode) async {
    await ref.read(appPreferencesProvider).writeThemeMode(mode);
    // Provider có thể đã bị huỷ trong lúc chờ ghi.
    if (!ref.mounted) return;
    state = AsyncData(mode);
  }
}
```

Với test, gói có sẵn `InMemorySharedPreferencesAsync`, bản nền tảng trong bộ nhớ không gọi plugin. Chưa gắn nền tảng nào thì constructor của `SharedPreferencesAsync` ném `StateError`; test cuối ghi lại điều đó.

### app_preferences_test.dart · 22-persistence-drift/test/app_preferences_test.dart (trích)

```dart
  // Cơ chế mock chính thức của SharedPreferencesAsync: thay bản cài đặt nền
  // tảng bằng bản trong bộ nhớ, không gọi plugin.
  void useStore([Map<String, Object> data = const {}]) {
    SharedPreferencesAsyncPlatform.instance =
        InMemorySharedPreferencesAsync.withData(data);
  }

  AppPreferences newPreferences() => AppPreferences(SharedPreferencesAsync());

  test('giá trị lạ trong kho thì quay về hệ thống, không ném lỗi', () async {
    useStore({AppPreferences.themeModeKey: 'solarized'});
    expect(await newPreferences().readThemeMode(), ThemeMode.system);
  });
```

## flutter_secure_storage và TokenStore

Token cho phép hành động như người dùng nên không thuộc về `shared_preferences`. `flutter_secure_storage` dùng Keychain trên iOS và macOS. Trên Android, với cấu hình mặc định, giá trị được mã hoá bằng AES-GCM; khoá AES đã bọc và bản mã nằm trong `SharedPreferences` của app, còn Android Keystore chỉ giữ khoá RSA dùng để bọc (README ghi RSA OAEP cộng AES-GCM; mã nguồn plugin gọi `AndroidKeyStore`). Ta che kho này sau một interface nhỏ:

### token_store.dart · 22-persistence-drift/lib/auth/token_store.dart (trích)

```dart
/// Nơi giữ token giữa các lần mở app. Phần còn lại của app chỉ biết interface
/// này, không biết Keychain hay Keystore.
abstract interface class TokenStore {
  /// Token đang lưu, hoặc `null` nếu chưa đăng nhập hoặc không đọc được.
  Future<AuthTokens?> read();

  Future<void> save(AuthTokens tokens);

  /// Xoá khi đăng xuất. Gọi nhiều lần vẫn an toàn.
  Future<void> clear();
}

/// Bản dùng `flutter_secure_storage`: Keychain trên iOS/macOS, khoá bọc trong
/// Android Keystore trên Android.
///
/// Cả hai token nằm trong **một** khoá dạng JSON, nên một lần ghi hoặc có đủ
/// cả hai hoặc không có gì; không bao giờ có access mới đi với refresh cũ.
class SecureTokenStore implements TokenStore {
  const SecureTokenStore(this._storage);

  final FlutterSecureStorage _storage;

  static const storageKey = 'auth.tokens';

  @override
  Future<AuthTokens?> read() async {
    final String? raw;
    try {
      raw = await _storage.read(key: storageKey);
    } on PlatformException {
      // Với AndroidOptions() mặc định (resetOnError: true), lỗi giải mã khi đọc
      // đã được plugin xử lý: xoá khoá hỏng, đọc lại và trả null. Còn lại là
      // các lỗi khác của nền tảng: coi như chưa đăng nhập để người dùng đăng
      // nhập lại, đừng làm app sập lúc khởi động.
      return null;
    }
    if (raw == null) return null;
    final tokens = _decode(raw);
    if (tokens == null) {
      // Dữ liệu hỏng thì xoá để lần sau khỏi đọc lại.
      await clear();
    }
    return tokens;
  }

  @override
  Future<void> save(AuthTokens tokens) => _storage.write(
    key: storageKey,
    value: jsonEncode({
      'access': tokens.accessToken,
      'refresh': tokens.refreshToken,
    }),
  );

  @override
  Future<void> clear() async {
    try {
      await _storage.delete(key: storageKey);
    } on PlatformException {
      // Đăng xuất không được thất bại vì kho lỗi; lần đọc sau đã trả null.
    }
  }

  // _decode: đọc JSON, sai dạng thì trả null
}
```

- **Một khoá JSON cho cả hai token**, để không có access mới đi với refresh cũ nếu lỗi giữa hai lần ghi.
- **`read` coi lỗi là chưa đăng nhập.** Với `AndroidOptions()` mặc định (`resetOnError: true`), lỗi giải mã khi đọc được plugin tự xử lý: xoá khoá hỏng, đọc lại và trả `null`, không ném (mã nguồn `FlutterSecureStorage.java`, `handleStorageError`). `PlatformException` còn lại là các lỗi khác; ta bắt và trả `null` để người dùng đăng nhập lại thay vì làm app sập lúc khởi động.
- **`clear` nuốt lỗi, `save` thì không**: đăng xuất không được thất bại vì kho lỗi, còn lưu thất bại thì màn hình phải biết.
- **`AuthTokens.toString` che giá trị** để token không lọt vào log.

Android cần thêm một bước ngoài code: README yêu cầu tắt Auto Backup (`android:allowBackup="false"`); lab không commit `android/` nên do người chạy tự làm. Về việc gỡ app, đây là kiến thức nền, chưa kiểm trên máy và Apple không cam kết: Keychain có thể còn dữ liệu sau khi gỡ app, còn Android mất khoá. Vì vậy lần mở app đầu sau khi cài lại nên gọi `clear()`. "Secure" không phải tuyệt đối: kho của hệ điều hành không cứu được thiết bị đã bị chiếm quyền (bài bảo mật mobile ở Phần 5 nói kỹ hơn).

Về test, `InMemoryTokenStore` dùng cho test màn hình, còn `SecureTokenStore` dùng `FlutterSecureStorage.setMockInitialValues` của gói. Một bộ test hợp đồng chạy chung cho cả hai bản, và nền tảng giả luôn báo lỗi để kiểm các nhánh `PlatformException`:

### token_store_test.dart · 22-persistence-drift/test/token_store_test.dart (trích)

```dart
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
```

Một test nữa không mock gì: `FlutterSecureStorage().read(...)` trên máy test ném `MissingPluginException`, nên `SecureTokenStore` chưa chạm Keychain hay Keystore thật trong lab này.

## Lỗi thường gặp

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Năm lỗi hay gặp khi lưu cục bộ</span>
<ul>
<li><strong>Sửa bảng mà không sinh lại mã.</strong> Dấu hiệu: analyzer báo thiếu <code>_$AppDatabase</code> hoặc cột mới. Cách sửa: <code>dart run build_runner build</code> rồi commit file sinh.</li>
<li><strong>Đổi schema mà thiếu bước migration.</strong> Dấu hiệu: test migration đỏ với <code>Schema does not match</code>; trên máy người dùng là lỗi cột không tồn tại. Cách sửa: tăng <code>schemaVersion</code>, <code>make-migrations</code>, viết <code>fromXToY</code>. Đừng sửa test cho khớp code.</li>
<li><strong>Ghi bằng SQL thô mà màn hình không đổi.</strong> Dấu hiệu: dữ liệu có trong file, stream đứng yên. Cách sửa: dùng API của drift, hoặc <code>customInsert(..., updates: {...})</code>, hoặc <code>notifyUpdates</code>.</li>
<li><strong>Test gọi plugin thật.</strong> Dấu hiệu: <code>MissingPluginException</code> hoặc <code>StateError</code> với <code>SharedPreferencesAsync</code>. Cách sửa: gắn nền tảng trong bộ nhớ do chính gói cung cấp.</li>
<li><strong>Để token ở chỗ dễ lộ.</strong> Dấu hiệu: token trong <code>shared_preferences</code>, log hay thông báo lỗi; Android khôi phục sao lưu gây <code>Failed to unwrap key</code>. Cách sửa: chỉ lưu qua <code>TokenStore</code>, che giá trị trong <code>toString</code>, tắt Auto Backup như README.</li>
</ul>
</div>

## Chạy ví dụ và kiểm chứng

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/22-persistence-drift
flutter pub get
flutter analyze
flutter test
```

Đã kiểm với Flutter 3.47.4, Dart 3.13.3: `flutter analyze` báo `No issues found!`, `flutter test` chạy **57 test** xanh từ thư mục sạch (xoá `.dart_tool` và `pubspec.lock`), không cần `build_runner`. Gói: `drift` 2.35.3, `drift_flutter` 0.3.1, `drift_dev` 2.35.2, `sqlite3` 3.9.0, `shared_preferences` 2.5.6, `flutter_secure_storage` 11.2.0, `flutter_riverpod` 3.5.0. Mức: **Kiểm chứng tự động**; khi `.dart_tool` chưa có bản tải sẵn cần mạng cho build hook.

| Test | Chứng minh |
|---|---|
| `db/trip_dao_test.dart` (13) | Upsert, `CHECK`, tên enum, thứ tự và `limit`, stream phát lại, `customStatement` và `customInsert` với stream |
| `trip_history_repository_test.dart` (10) | `Trip` ghi rồi đọc ra đúng; mét, đồng, UTC; dòng hỏng |
| `drift/.../migration_test.dart` (2) | Schema 1 lên 2 khớp kỳ vọng, dữ liệu cũ sống sót |
| `migration_from_old_file_test.dart` (3) | Database dựng tay được nâng cấp; hạ phiên bản ném `StateError` |
| `app_preferences_test.dart` (5) | Mặc định, ghi/đọc, giá trị lạ, chưa gắn nền tảng |
| `token_store_test.dart` (18) | Hợp đồng chung hai `TokenStore`, JSON hỏng, kho lỗi, `toString` che token |
| `app_test.dart` (6) | Màn hình lịch sử, stream cập nhật, dòng hỏng, sáng/tối còn sau khi mở lại, đăng nhập/đăng xuất |

**Chưa làm**: chạy trên iOS Simulator, Android emulator hay thiết bị thật (Keychain, Keystore, isolate nền và đường dẫn file thật chưa kiểm); web và mã hoá file SQLite; xử lý hạ phiên bản; đồng bộ với server và bảng outbox (bài 46, Phần 6); `sqlite3` trên Linux CI.

## Bài tập

1. Thêm cột `rating` (số nguyên, có thể chưa có) vào `Trips` ở schema 3. Liệt kê các bước và viết thân hàm `from2To3`.
2. Code đồng bộ chèn chuyến bằng `db.customStatement('INSERT ...')` và màn hình không cập nhật. Nêu hai cách sửa mà không đổi sang API ghi của drift.
3. Vì sao `SecureTokenStore.clear()` nuốt `PlatformException` còn `save()` để nó đi qua?

<details><summary>Xem lời giải</summary>
<p><strong>1.</strong> Thêm <code>IntColumn get rating =&gt; integer().nullable()();</code> vào cuối <code>Trips</code>, tăng <code>schemaVersion</code> lên 3, chạy <code>build_runner</code> rồi <code>make-migrations</code>. Thân hàm: <code>await m.addColumn(schema.trips, schema.trips.rating);</code>; cột nullable nên không cần mặc định. Rồi chạy lại test migration.</p>
<p><strong>2.</strong> Dùng <code>customInsert(sql, updates: {db.trips})</code> thay cho <code>customStatement</code>, hoặc giữ lệnh cũ rồi gọi <code>db.notifyUpdates({TableUpdate.onTable(db.trips, kind: UpdateKind.insert)})</code>. Cả hai có test trong <code>trip_dao_test.dart</code>.</p>
<p><strong>3.</strong> Đăng xuất phải thành công từ góc nhìn người dùng, và nếu kho lỗi thì <code>read</code> sau đó vẫn trả <code>null</code>. Còn <code>save()</code> thất bại nghĩa là phiên sẽ mất ở lần mở sau mà người dùng không biết, nên màn hình phải biết để báo và cho thử lại.</p>
</details>

Bài trước: [Clean Architecture cho mobile: domain, data, presentation và repository](/articles/flutter-clean-architecture-domain-data-presentation-mobile). Bài tiếp: [go_router: deep link, ShellRoute, guard đăng nhập](/articles/flutter-go-router-deep-link-shellroute-guard).

## Tài liệu gốc

- [Setup](https://drift.simonbinder.eu/setup/), [Stream queries](https://drift.simonbinder.eu/dart_api/streams/), [Migrations](https://drift.simonbinder.eu/migrations/), [Testing migrations](https://drift.simonbinder.eu/migrations/tests/) và [Testing](https://drift.simonbinder.eu/testing/) (drift.simonbinder.eu).
- [drift](https://pub.dev/packages/drift), [drift_flutter](https://pub.dev/packages/drift_flutter), [sqlite3](https://pub.dev/packages/sqlite3), [shared_preferences](https://pub.dev/packages/shared_preferences), [flutter_secure_storage](https://pub.dev/packages/flutter_secure_storage) và [path_provider](https://pub.dev/packages/path_provider) (pub.dev).
- [Keychain services](https://developer.apple.com/documentation/security/keychain-services) (developer.apple.com).
- [Android Keystore system](https://developer.android.com/privacy-and-security/keystore) và [Auto Backup](https://developer.android.com/guide/topics/data/autobackup) (developer.android.com).
