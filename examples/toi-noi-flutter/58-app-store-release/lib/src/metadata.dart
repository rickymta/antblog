import 'dart:io';

/// Metadata của một app theo bố cục thư mục mà `fastlane deliver` đọc
/// (`metadata_path`): file `.txt` theo từng ngôn ngữ, thư mục
/// `review_information/` và vài file không theo ngôn ngữ.
///
/// Trường thiếu file là `null`; file rỗng là chuỗi rỗng. Hai trường hợp đó
/// được báo bằng hai mã khác nhau trong `ReleaseChecklist`.
final class LocalizedMetadata {
  const LocalizedMetadata({
    required this.locale,
    this.name,
    this.subtitle,
    this.keywords,
    this.description,
    this.promotionalText,
    this.releaseNotes,
    this.supportUrl,
    this.marketingUrl,
    this.privacyUrl,
  });

  final String locale;
  final String? name;
  final String? subtitle;
  final String? keywords;
  final String? description;
  final String? promotionalText;
  final String? releaseNotes;
  final String? supportUrl;
  final String? marketingUrl;
  final String? privacyUrl;
}

/// Mục "App Review Information" của App Store Connect.
final class ReviewInformation {
  const ReviewInformation({
    this.firstName,
    this.lastName,
    this.phoneNumber,
    this.emailAddress,
    this.demoUser,
    this.demoPassword,
    this.notes,
  });

  final String? firstName;
  final String? lastName;
  final String? phoneNumber;
  final String? emailAddress;
  final String? demoUser;
  final String? demoPassword;
  final String? notes;
}

final class AppMetadata {
  const AppMetadata({
    required this.locales,
    required this.review,
    this.copyright,
    this.primaryCategory,
    this.secondaryCategory,
  });

  /// Đọc thư mục [dir] (thư mục `metadata` của deliver).
  factory AppMetadata.read(Directory dir) {
    String? text(String path) {
      final f = File('${dir.path}/$path');
      return f.existsSync() ? f.readAsStringSync().trimRight() : null;
    }

    final locales = <LocalizedMetadata>[];
    for (final entity in dir.listSync().whereType<Directory>()) {
      final locale = entity.uri.pathSegments.where((s) => s.isNotEmpty).last;
      if (locale == 'review_information') continue;
      String? t(String file) => text('$locale/$file');
      locales.add(
        LocalizedMetadata(
          locale: locale,
          name: t('name.txt'),
          subtitle: t('subtitle.txt'),
          keywords: t('keywords.txt'),
          description: t('description.txt'),
          promotionalText: t('promotional_text.txt'),
          releaseNotes: t('release_notes.txt'),
          supportUrl: t('support_url.txt'),
          marketingUrl: t('marketing_url.txt'),
          privacyUrl: t('privacy_url.txt'),
        ),
      );
    }
    String? r(String file) => text('review_information/$file');
    return AppMetadata(
      locales: locales,
      review: ReviewInformation(
        firstName: r('first_name.txt'),
        lastName: r('last_name.txt'),
        phoneNumber: r('phone_number.txt'),
        emailAddress: r('email_address.txt'),
        demoUser: r('demo_user.txt'),
        demoPassword: r('demo_password.txt'),
        notes: r('notes.txt'),
      ),
      copyright: text('copyright.txt'),
      primaryCategory: text('primary_category.txt'),
      secondaryCategory: text('secondary_category.txt'),
    );
  }

  final List<LocalizedMetadata> locales;
  final ReviewInformation review;
  final String? copyright;
  final String? primaryCategory;
  final String? secondaryCategory;
}
