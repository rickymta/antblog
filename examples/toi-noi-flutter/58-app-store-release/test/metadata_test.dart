import 'dart:convert';

import 'package:test/test.dart';
import 'package:toi_noi_58/toi_noi_58.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'support.dart';

/// Phát hiện của checklist khi metadata của app khách được sửa bằng [edit].
Set<String> metaCodes(
  AppMetadata Function(AppMetadata) edit, {
  bool isFirstVersion = true,
}) {
  final base = loadSample(StoreApp.customer, isFirstVersion: isFirstVersion);
  return codes(
    ReleaseChecklist(change(base, metadata: edit(base.metadata))).run(),
  );
}

Set<String> onlyMeta(Set<String> all) => {
  for (final c in all)
    if (!c.startsWith('review-')) c,
};

void main() {
  group('gói mẫu của hai app', () {
    for (final app in StoreApp.values) {
      test('${app.name} (${app.bundleId}): không có phát hiện nào', () {
        expect(ReleaseChecklist(loadSample(app)).run(), isEmpty);
      });
    }

    test('Bundle ID khớp curriculum §3.2', () {
      expect(StoreApp.customer.bundleId, 'vn.toinoi.customer');
      expect(StoreApp.driver.bundleId, 'vn.toinoi.driver');
    });

    test(
      'bản cập nhật (không phải phiên bản đầu) thiếu release notes là lỗi',
      () {
        final input = loadSample(StoreApp.customer, isFirstVersion: false);
        expect(errorCodes(ReleaseChecklist(input).run()), {
          'release-notes-missing',
        });
      },
    );

    test('phiên bản đầu có release notes chỉ là cảnh báo', () {
      expect(metaCodes((m) => withLocale(m, releaseNotes: 'Sửa lỗi nhỏ.')), {
        'release-notes-first-version',
      });
    });

    test('ghi chú reviewer ghi cước 5 km khớp calculateFare, không gõ tay', () {
      final fare = calculateFare(5).toString();
      expect(fare, '25.500 VND');
      for (final app in StoreApp.values) {
        expect(loadSample(app).metadata.review.notes, contains(fare));
      }
    });
  });

  group('tên và subtitle (đếm ký tự)', () {
    test('tên 30 ký tự được, 31 bị lỗi, 2 được, 1 bị lỗi', () {
      expect(metaCodes((m) => withLocale(m, name: 'a' * 30)), isEmpty);
      expect(metaCodes((m) => withLocale(m, name: 'a' * 31)), {'name-length'});
      expect(metaCodes((m) => withLocale(m, name: 'ab')), isEmpty);
      expect(metaCodes((m) => withLocale(m, name: 'z')), {'name-length'});
    });

    test('đếm theo ký tự, không theo byte: 30 chữ ớ (90 byte) vẫn hợp lệ', () {
      final name = 'ớ' * 30;
      expect(utf8.encode(name).length, 90);
      expect(metaCodes((m) => withLocale(m, name: name)), isEmpty);
    });

    test('subtitle 30 ký tự được, 31 bị lỗi', () {
      expect(metaCodes((m) => withLocale(m, subtitle: 'b' * 30)), isEmpty);
      expect(metaCodes((m) => withLocale(m, subtitle: 'b' * 31)), {
        'subtitle-length',
      });
    });

    test('subtitle khẳng định không kiểm chứng được bị cảnh báo', () {
      expect(metaCodes((m) => withLocale(m, subtitle: 'Gọi xe rẻ nhất')), {
        'subtitle-claim',
      });
    });

    test(
      'dạng NFD: tên 16 chữ ớ thành 48 điểm mã, vượt 30 và bị cảnh báo NFC',
      () {
        // "ớ" dạng NFD = o + dấu móc (U+031B) + dấu sắc (U+0301).
        final nfd = 'ớ' * 16;
        expect(metaCodes((m) => withLocale(m, name: nfd)), {
          'name-length',
          'text-not-nfc',
        });
      },
    );
  });

  group('keywords (đếm byte UTF-8)', () {
    test('100 byte ASCII được, 101 bị lỗi', () {
      final ok = '${'a' * 49},${'b' * 50}';
      expect(utf8.encode(ok).length, 100);
      expect(metaCodes((m) => withLocale(m, keywords: ok)), isEmpty);
      expect(metaCodes((m) => withLocale(m, keywords: '${ok}c')), {
        'keywords-bytes',
      });
    });

    test('34 chữ ớ chỉ 34 ký tự nhưng 102 byte: vượt giới hạn byte', () {
      final kw = 'ớ' * 34;
      expect(kw.runes.length, 34);
      expect(utf8.encode(kw).length, 102);
      expect(metaCodes((m) => withLocale(m, keywords: kw)), {'keywords-bytes'});
    });

    test('33 chữ ớ là 99 byte, hợp lệ', () {
      expect(metaCodes((m) => withLocale(m, keywords: 'ớ' * 33)), isEmpty);
    });

    test('từ khoá phải dài hơn hai ký tự: "ôm" lỗi, "xôm" được', () {
      expect(metaCodes((m) => withLocale(m, keywords: 'ôm,taxi')), {
        'keyword-too-short',
      });
      expect(metaCodes((m) => withLocale(m, keywords: 'xôm,taxi')), isEmpty);
    });

    test('từ khoá lặp tên app là lỗi, không phân biệt hoa thường', () {
      expect(metaCodes((m) => withLocale(m, keywords: 'TỚI NƠI,taxi')), {
        'keyword-repeats-name',
      });
    });

    test('từ khoá lặp và dấu phẩy thừa là cảnh báo', () {
      expect(metaCodes((m) => withLocale(m, keywords: 'taxi,Taxi')), {
        'keyword-duplicate',
      });
      expect(metaCodes((m) => withLocale(m, keywords: 'taxi,,xe ôm')), {
        'keywords-empty-item',
      });
    });
  });

  group('promotional text, description, release notes', () {
    test('promotional text 170 được, 171 lỗi', () {
      expect(
        metaCodes((m) => withLocale(m, promotionalText: 'p' * 170)),
        isEmpty,
      );
      expect(metaCodes((m) => withLocale(m, promotionalText: 'p' * 171)), {
        'promo-length',
      });
    });

    test(
      'description 4000 ký tự được (kể cả khi lớn hơn 4000 byte), 4001 lỗi',
      () {
        expect(
          metaCodes((m) => withLocale(m, description: 'ớ' * 4000)),
          isEmpty,
        );
        expect(metaCodes((m) => withLocale(m, description: 'a' * 4001)), {
          'description-length',
        });
      },
    );

    test('release notes 4000 được, 4001 lỗi (bản cập nhật)', () {
      expect(
        metaCodes(
          (m) => withLocale(m, releaseNotes: 'r' * 4000),
          isFirstVersion: false,
        ),
        isEmpty,
      );
      expect(
        metaCodes(
          (m) => withLocale(m, releaseNotes: 'r' * 4001),
          isFirstVersion: false,
        ),
        {'release-notes-length'},
      );
    });
  });

  group('thiếu file, nội dung tạm, URL', () {
    test('file thiếu và file rỗng báo hai mã khác nhau', () {
      final base = loadSample(StoreApp.customer);
      final onlyName = AppMetadata(
        locales: const [LocalizedMetadata(locale: 'vi', name: 'Tới Nơi')],
        review: base.metadata.review,
        copyright: base.metadata.copyright,
        primaryCategory: base.metadata.primaryCategory,
      );
      expect(
        onlyMeta(
          codes(ReleaseChecklist(change(base, metadata: onlyName)).run()),
        ),
        {
          'keywords-missing',
          'description-missing',
          'support-url-missing',
          'privacy-url-missing',
        },
      );
      expect(metaCodes((m) => withLocale(m, keywords: '')), {
        'keywords-missing-empty',
      });
    });

    test('không có thư mục ngôn ngữ nào là lỗi', () {
      final base = loadSample(StoreApp.customer);
      final none = AppMetadata(
        locales: const [],
        review: base.metadata.review,
        copyright: base.metadata.copyright,
        primaryCategory: base.metadata.primaryCategory,
      );
      expect(codes(ReleaseChecklist(change(base, metadata: none)).run()), {
        'metadata-no-locale',
      });
    });

    test('nội dung tạm bị chặn ở mọi trường văn bản (2.1(a))', () {
      for (final text in ['TODO viết mô tả', 'Lorem ipsum dolor', 'điền sau']) {
        expect(metaCodes((m) => withLocale(m, description: text)), {
          'placeholder-text',
        }, reason: text);
      }
      expect(metaCodes((m) => withLocale(m, promotionalText: 'TBD')), {
        'placeholder-text',
      });
    });

    test('URL phải đầy đủ giao thức; http chỉ là cảnh báo', () {
      expect(
        metaCodes((m) => withLocale(m, supportUrl: 'toinoi.example/ho-tro')),
        {'url-invalid'},
      );
      expect(
        metaCodes((m) => withLocale(m, privacyUrl: 'ftp://toinoi.example/p')),
        {'url-invalid'},
      );
      expect(
        metaCodes((m) => withLocale(m, marketingUrl: 'http://toinoi.example')),
        {'url-not-https'},
      );
    });

    test(
      'copyright: "2026 Chủ sở hữu" được, "© 2026 ..." và thiếu năm bị lỗi',
      () {
        expect(
          metaCodes((m) => withLocale(m, copyright: '2026 Công ty Tới Nơi')),
          isEmpty,
        );
        expect(
          metaCodes((m) => withLocale(m, copyright: '© 2026 Công ty Tới Nơi')),
          {'copyright-format'},
        );
        expect(metaCodes((m) => withLocale(m, copyright: 'Công ty Tới Nơi')), {
          'copyright-format',
        });
      },
    );

    test('danh mục phải có trong danh sách của deliver', () {
      expect(metaCodes((m) => withLocale(m, primaryCategory: 'TAXI')), {
        'category-primary',
      });
      expect(metaCodes((m) => withLocale(m, secondaryCategory: 'Du lịch')), {
        'category-secondary',
      });
    });
  });

  group('App Review Information', () {
    ReviewInformation review(ReleaseInput i) => i.metadata.review;

    Set<String> reviewCodes(
      ReviewInformation Function(ReviewInformation) edit, {
      StoreApp app = StoreApp.customer,
    }) {
      final base = loadSample(app);
      return codes(
        ReleaseChecklist(
          change(
            base,
            metadata: withLocale(base.metadata, review: edit(review(base))),
          ),
        ).run(),
      );
    }

    test('số điện thoại phải dạng quốc tế có dấu +', () {
      expect(reviewCodes((r) => withReview(r, phoneNumber: '0900000000')), {
        'review-phone-format',
      });
      expect(
        reviewCodes((r) => withReview(r, phoneNumber: '+84 900 000 000')),
        isEmpty,
      );
    });

    test('thiếu demo account hoặc liên hệ là lỗi', () {
      expect(reviewCodes((r) => withReview(r, demoUser: '')), {
        'review-demo-missing',
      });
      expect(reviewCodes((r) => withReview(r, demoPassword: ' ')), {
        'review-demo-missing',
      });
      expect(reviewCodes((r) => withReview(r, firstName: '')), {
        'review-contact-missing',
      });
    });

    test('ghi chú 4000 byte được, 4001 byte lỗi; chữ ớ 3 byte nên vượt dù ít hơn 4000 ký tự', () {
      const lead = 'xoá tài khoản ';
      final leadBytes = utf8.encode(lead).length;
      String notesOf(int bytes) => lead + 'a' * (bytes - leadBytes);
      expect(utf8.encode(notesOf(4000)).length, 4000);
      expect(reviewCodes((r) => withReview(r, notes: notesOf(4000))), isEmpty);
      expect(reviewCodes((r) => withReview(r, notes: notesOf(4001))), {
        'review-notes-bytes',
      });
      final vn = lead + 'ớ' * 1330;
      expect(vn.runes.length, lessThan(4000));
      expect(utf8.encode(vn).length, greaterThan(4000));
      expect(reviewCodes((r) => withReview(r, notes: vn)), {
        'review-notes-bytes',
      });
    });

    test('ghi chú rỗng là lỗi; thiếu chỗ xoá tài khoản là cảnh báo', () {
      expect(reviewCodes((r) => withReview(r, notes: '')), {
        'review-notes-missing',
      });
      expect(
        reviewCodes(
          (r) => withReview(r, notes: 'Đăng nhập bằng tài khoản demo.'),
        ),
        {'review-notes-account-deletion'},
      );
    });

    test(
      'app tài xế dùng vị trí nền mà ghi chú không giải thích bị cảnh báo',
      () {
        expect(
          reviewCodes(
            (r) => withReview(r, notes: 'Hồ sơ > Xoá tài khoản.'),
            app: StoreApp.driver,
          ),
          {'review-notes-background-location'},
        );
      },
    );

    test(
      'ghi chú app tài xế thật có nói vị trí nền; app khách nói không dùng nền',
      () {
        expect(
          loadSample(StoreApp.driver).metadata.review.notes,
          contains('Vị trí nền'),
        );
        expect(
          loadSample(StoreApp.customer).metadata.review.notes,
          contains('không dùng vị trí nền'),
        );
      },
    );
  });

  group('IPHONEOS_DEPLOYMENT_TARGET', () {
    test('iOS 13 được, thấp hơn bị lỗi', () {
      final base = loadSample(StoreApp.customer);
      expect(
        ReleaseChecklist(change(base, deploymentTarget: 13)).run(),
        isEmpty,
      );
      expect(
        codes(ReleaseChecklist(change(base, deploymentTarget: 12)).run()),
        {'deployment-target-low'},
      );
    });
  });
}
