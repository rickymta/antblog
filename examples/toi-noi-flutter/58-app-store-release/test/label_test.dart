import 'package:test/test.dart';
import 'package:toi_noi_58/toi_noi_58.dart';

import 'support.dart';

const _phone = ManifestVocabulary.phoneNumber;
const _precise = ManifestVocabulary.preciseLocation;
const _coarse = ManifestVocabulary.coarseLocation;
const _user = ManifestVocabulary.userId;
const _purchase = ManifestVocabulary.purchaseHistory;
const _device = ManifestVocabulary.deviceId;
const _crash = ManifestVocabulary.crashData;
const _fn = ManifestVocabulary.appFunctionality;
const _analytics = ManifestVocabulary.analytics;

const _whenInUse = 'NSLocationWhenInUseUsageDescription';
const _always = 'NSLocationAlwaysAndWhenInUseUsageDescription';
const _longText =
    'Tới Nơi dùng vị trí của bạn khi đang mở app để đặt ghim điểm đón.';

/// Checklist trên app khách với label, Info.plist hoặc manifest được thay.
Set<String> run({
  PrivacyLabel? label,
  InfoPlist? info,
  PrivacyManifest? manifest,
  StoreApp app = StoreApp.customer,
}) {
  final base = loadSample(app);
  return codes(
    ReleaseChecklist(change(base, label: label, info: info, manifest: manifest))
        .run(),
  );
}

/// Label mẫu của app khách, bỏ các loại trong [without] rồi thêm [extra].
PrivacyLabel customerLabel({
  Set<String> without = const {},
  List<Map<String, Object?>> extra = const [],
  bool tracking = false,
}) {
  final entries = [
    labelEntry(_phone),
    labelEntry(_precise),
    labelEntry(_user),
    labelEntry(_purchase),
    labelEntry(_device, source: 'sdk:firebase_messaging'),
    labelEntry(_crash, source: 'sdk:firebase_crashlytics', linked: false),
  ];
  return labelOf([
    for (final e in entries)
      if (!without.contains(e['type'])) e,
    ...extra,
  ], tracking: tracking);
}

PrivacyManifest manifestWith(List<String> types, {bool tracking = false}) =>
    PrivacyManifest.parse(
      manifestXml(
        tracking: tracking,
        domains: tracking ? ['ads.toinoi.example'] : const [],
        collected: [
          for (final t in types)
            (type: t, linked: true, tracking: false, purposes: [_fn]),
        ],
      ),
    );

void main() {
  group('label thật khớp manifest và Info.plist', () {
    for (final app in StoreApp.values) {
      test('${app.name}: không có phát hiện nào', () {
        expect(ReleaseChecklist(loadSample(app)).run(), isEmpty);
      });
    }

    test('dòng của SDK (DeviceID, CrashData) có trong label mà không có trong manifest của app', () {
      final input = loadSample(StoreApp.customer);
      final sdkTypes = {
        for (final e in input.label.entries)
          if (!e.fromApp) e.data.type,
      };
      expect(sdkTypes, {_device, _crash});
      expect(input.manifest.collectedTypes.intersection(sdkTypes), isEmpty);
    });

    test('mọi dòng nguồn app của label có trong manifest và ngược lại', () {
      for (final app in StoreApp.values) {
        final input = loadSample(app);
        final appTypes = {
          for (final e in input.label.entries)
            if (e.fromApp) e.data.type,
        };
        expect(appTypes, input.manifest.collectedTypes, reason: app.name);
      }
    });
  });

  group('label so với manifest', () {
    test('manifest có loại mà label chưa khai', () {
      expect(run(label: customerLabel(without: {_purchase})), {
        'label-missing-type',
      });
    });

    test('label khai loại nguồn app mà manifest không có', () {
      expect(
        run(label: customerLabel(extra: [labelEntry(ManifestVocabulary.name)])),
        {'label-app-type-not-in-manifest'},
      );
    });

    test('cùng loại nhưng linked khác nhau', () {
      final label = labelOf([
        labelEntry(_phone, linked: false),
        labelEntry(_precise),
        labelEntry(_user),
        labelEntry(_purchase),
      ]);
      expect(run(label: label), {'label-manifest-mismatch'});
    });

    test('label thiếu một mục đích mà manifest đã khai', () {
      final manifest = manifestWith([_phone, _precise, _user, _purchase]);
      final label = labelOf([
        labelEntry(
          _phone,
          purposes: ['NSPrivacyCollectedDataTypePurposeOther'],
        ),
        labelEntry(_precise),
        labelEntry(_user),
        labelEntry(_purchase),
      ]);
      expect(run(label: label, manifest: manifest), {
        'label-manifest-mismatch',
      });
    });

    test(
      'label có thêm mục đích so với manifest vẫn khớp (label gồm cả SDK)',
      () {
        final label = labelOf([
          labelEntry(_phone, purposes: [_fn, _analytics]),
          labelEntry(_precise),
          labelEntry(_user),
          labelEntry(_purchase),
        ]);
        expect(run(label: label), isEmpty);
      },
    );

    test('tracking của label khác manifest', () {
      expect(run(label: customerLabel(tracking: true)), {
        'label-tracking-mismatch',
        'tracking-without-att-description',
      });
    });
  });

  group('label so với mô tả quyền trong Info.plist', () {
    test('xin quyền vị trí mà label không khai vị trí', () {
      final codes = run(
        label: customerLabel(without: {_precise}),
        manifest: manifestWith([_phone, _user, _purchase]),
      );
      expect(codes, {'label-missing-location'});
    });

    test('Coarse Location cũng đủ để khớp với quyền vị trí', () {
      final label = labelOf([
        labelEntry(_phone),
        labelEntry(_coarse),
        labelEntry(_user),
        labelEntry(_purchase),
      ]);
      final manifest = manifestWith([_phone, _coarse, _user, _purchase]);
      expect(run(label: label, manifest: manifest), isEmpty);
    });

    test('label khai vị trí mà Info.plist không có mô tả quyền', () {
      final info = InfoPlist.parse(infoXml());
      expect(run(info: info), {'location-without-usage-description'});
    });

    test('UIBackgroundModes location mà không có mô tả quyền vị trí', () {
      final info = InfoPlist.parse(infoXml(backgroundModes: ['location']));
      expect(run(info: info), {
        'location-without-usage-description',
        'background-location-without-usage',
      });
    });

    test('quyền "luôn cho phép" mà không có background mode là cảnh báo', () {
      final info = InfoPlist.parse(
        infoXml(usage: {_whenInUse: _longText, _always: _longText}),
      );
      expect(run(info: info), {'always-without-background-mode'});
    });

    test('mô tả quyền quá ngắn là cảnh báo, còn nội dung tạm là lỗi', () {
      expect(
        run(info: InfoPlist.parse(infoXml(usage: {_whenInUse: 'Cần vị trí'}))),
        {'usage-description-short'},
      );
      final placeholder = run(
        info: InfoPlist.parse(
          infoXml(usage: {_whenInUse: 'TODO viết mô tả quyền vị trí sau'}),
        ),
      );
      expect(placeholder, {'usage-description-placeholder'});
    });

    test('quyền liên hệ, micro, ảnh, camera gợi ý khai kiểu dữ liệu tương ứng (cảnh báo)', () {
      final info = InfoPlist.parse(
        infoXml(
          usage: {
            _whenInUse: _longText,
            'NSContactsUsageDescription':
                'Tới Nơi đọc danh bạ để gợi ý người nhận bưu kiện.',
            'NSMicrophoneUsageDescription':
                'Tới Nơi ghi âm khi bạn gọi cho tài xế trong app.',
            'NSCameraUsageDescription':
                'Tới Nơi chụp ảnh gói hàng để làm bằng chứng giao nhận.',
          },
        ),
      );
      final found = ReleaseChecklist(
        change(loadSample(StoreApp.customer), info: info),
      ).run();
      expect(codes(found), {'label-missing-for-usage'});
      expect(
        found.where((f) => f.code == 'label-missing-for-usage'),
        hasLength(3),
      );
      expect(found.every((f) => f.severity == Severity.warning), isTrue);
    });

    test('khai tracking thì cần mô tả quyền ATT; có ATT mà không tracking là cảnh báo', () {
      final withAtt = InfoPlist.parse(
        infoXml(
          usage: {
            _whenInUse: _longText,
            'NSUserTrackingUsageDescription': 'Tới Nơi dùng mã định danh để đo hiệu quả quảng cáo của chính Tới Nơi.',
          },
        ),
      );
      expect(run(info: withAtt), {'att-description-without-tracking'});

      final manifest = manifestWith([
        _phone,
        _precise,
        _user,
        _purchase,
      ], tracking: true);
      expect(run(label: customerLabel(tracking: true), manifest: manifest), {
        'tracking-without-att-description',
      });
      expect(
        run(
          label: customerLabel(tracking: true),
          manifest: manifest,
          info: withAtt,
        ),
        isEmpty,
      );
    });
  });

  group('app tài xế', () {
    test('có quyền "luôn cho phép" và background mode location, khớp nhau', () {
      final info = loadSample(StoreApp.driver).info;
      expect(info.usageDescriptions.keys, contains(_always));
      expect(info.usesBackgroundLocation, isTrue);
      expect(info.backgroundModes, ['location', 'remote-notification']);
    });

    test('bỏ UIBackgroundModes thì quyền "luôn cho phép" thành cảnh báo', () {
      final info = InfoPlist.parse(
        infoXml(usage: loadSample(StoreApp.driver).info.usageDescriptions),
      );
      expect(run(app: StoreApp.driver, info: info), {
        'always-without-background-mode',
      });
    });

    test(
      'app khách không có quyền "luôn cho phép" và không có background mode',
      () {
        final info = loadSample(StoreApp.customer).info;
        expect(info.usageDescriptions.keys, [_whenInUse]);
        expect(info.backgroundModes, isEmpty);
      },
    );
  });

  group('đọc file', () {
    test('label sai cấu trúc ném FormatException', () {
      expect(() => PrivacyLabel.parse('[]'), throwsFormatException);
      expect(
        () => PrivacyLabel.parse('{"tracking": false}'),
        throwsFormatException,
      );
    });

    test('Info.plist: chỉ lấy khoá NS...UsageDescription có giá trị chuỗi', () {
      final info = InfoPlist.parse(
        infoXml(usage: {_whenInUse: _longText, 'CFBundleName': 'Tới Nơi'}),
      );
      expect(info.usageDescriptions.keys, [_whenInUse]);
    });
  });
}
