/// Giới hạn và danh sách giá trị lấy từ trang chính thức của Apple, kiểm tại
/// 11/2026. Apple đổi chúng theo thời gian: mỗi hằng ghi nguồn để kiểm lại.
library;

/// Giới hạn của trường metadata trong App Store Connect.
///
/// Nguồn: "App information" và "Platform version information" (App Store
/// Connect Help, mục Reference), App Review Guidelines 2.3.7.
abstract final class StoreLimits {
  static const int nameMin = 2;
  static const int nameMax = 30;
  static const int subtitleMax = 30;

  /// Tính bằng **byte** (UTF-8), không phải ký tự: chữ có dấu tiếng Việt
  /// chiếm 2 hoặc 3 byte.
  static const int keywordsMaxBytes = 100;

  /// Mỗi từ khoá phải dài hơn hai ký tự.
  static const int keywordMinChars = 3;
  static const int promotionalTextMax = 170;
  static const int descriptionMax = 4000;
  static const int releaseNotesMax = 4000;

  /// Ghi chú cho App Review: 4.000 **byte**, viết bằng ngôn ngữ nào cũng được.
  static const int reviewNotesMaxBytes = 4000;

  static const int screenshotsMaxPerSize = 10;
}

/// Kích thước ảnh chụp (px, chiều rộng x chiều cao theo hướng dọc).
///
/// Nguồn: "Screenshot specifications" (App Store Connect Help). Đây là các
/// kích thước iPhone và iPad lab biết; trang Apple còn liệt kê thêm.
abstract final class ScreenshotSizes {
  /// iPhone có Dynamic Island, màn hình medium: bắt buộc có ít nhất một ảnh.
  static const List<(int, int)> iphoneRequired = [(1179, 2556), (1206, 2622)];

  /// Các cỡ iPhone khác mà trang Apple chấp nhận.
  static const List<(int, int)> iphoneOther = [
    (1260, 2736),
    (1290, 2796),
    (1320, 2868),
    (1284, 2778),
    (1242, 2688),
    (1170, 2532),
    (1125, 2436),
    (1080, 2340),
    (1242, 2208),
    (750, 1334),
  ];

  /// iPad 13 inch: bắt buộc nếu app chạy được trên iPad.
  static const List<(int, int)> ipadRequired = [(2064, 2752), (2048, 2732)];

  static const List<(int, int)> ipadOther = [
    (1488, 2266),
    (1512, 2316),
    (1668, 2420),
    (1668, 2388),
    (1640, 2360),
    (1668, 2224),
  ];

  /// Cả hướng dọc và ngang: ảnh ngang đảo hai số.
  static bool matches(List<(int, int)> sizes, int width, int height) =>
      sizes.any(
        (s) =>
            (s.$1 == width && s.$2 == height) ||
            (s.$1 == height && s.$2 == width),
      );
}

/// Danh mục required reason API và lý do hợp lệ.
///
/// Nguồn: "Describing use of required reason API" và
/// `NSPrivacyAccessedAPITypeReasons` (developer.apple.com/documentation).
abstract final class RequiredReasonApi {
  static const String fileTimestamp =
      'NSPrivacyAccessedAPICategoryFileTimestamp';
  static const String systemBootTime =
      'NSPrivacyAccessedAPICategorySystemBootTime';
  static const String diskSpace = 'NSPrivacyAccessedAPICategoryDiskSpace';
  static const String activeKeyboards =
      'NSPrivacyAccessedAPICategoryActiveKeyboards';
  static const String userDefaults = 'NSPrivacyAccessedAPICategoryUserDefaults';

  static const Map<String, Set<String>> reasonsByCategory = {
    fileTimestamp: {'DDA9.1', 'C617.1', '3B52.1', '0A2A.1'},
    systemBootTime: {'35F9.1', '8FFB.1', '3D61.1'},
    diskSpace: {'85F4.1', 'E174.1', '7D9E.1', 'B728.1'},
    activeKeyboards: {'3EC4.1', '54BD.1'},
    userDefaults: {'CA92.1', '1C8F.1', 'C56D.1', 'AC6B.1'},
  };

  /// Chỉ SDK bên thứ ba được khai: bọc API cho app dùng.
  static const Set<String> sdkOnlyReasons = {'0A2A.1', 'C56D.1'};
}

/// Danh mục kiểu dữ liệu và mục đích trong privacy manifest.
///
/// Nguồn: `NSPrivacyCollectedDataType` và `NSPrivacyCollectedDataTypePurposes`.
abstract final class ManifestVocabulary {
  static const String preciseLocation =
      'NSPrivacyCollectedDataTypePreciseLocation';
  static const String coarseLocation =
      'NSPrivacyCollectedDataTypeCoarseLocation';
  static const String phoneNumber = 'NSPrivacyCollectedDataTypePhoneNumber';
  static const String name = 'NSPrivacyCollectedDataTypeName';
  static const String userId = 'NSPrivacyCollectedDataTypeUserID';
  static const String deviceId = 'NSPrivacyCollectedDataTypeDeviceID';
  static const String contacts = 'NSPrivacyCollectedDataTypeContacts';
  static const String audioData = 'NSPrivacyCollectedDataTypeAudioData';
  static const String photosOrVideos =
      'NSPrivacyCollectedDataTypePhotosorVideos';
  static const String purchaseHistory =
      'NSPrivacyCollectedDataTypePurchaseHistory';
  static const String crashData = 'NSPrivacyCollectedDataTypeCrashData';
  static const String performanceData =
      'NSPrivacyCollectedDataTypePerformanceData';

  static const Set<String> dataTypes = {
    'NSPrivacyCollectedDataTypeName',
    'NSPrivacyCollectedDataTypeEmailAddress',
    'NSPrivacyCollectedDataTypePhoneNumber',
    'NSPrivacyCollectedDataTypePhysicalAddress',
    'NSPrivacyCollectedDataTypeOtherUserContactInfo',
    'NSPrivacyCollectedDataTypeHealth',
    'NSPrivacyCollectedDataTypeFitness',
    'NSPrivacyCollectedDataTypePaymentInfo',
    'NSPrivacyCollectedDataTypeCreditInfo',
    'NSPrivacyCollectedDataTypeOtherFinancialInfo',
    'NSPrivacyCollectedDataTypePreciseLocation',
    'NSPrivacyCollectedDataTypeCoarseLocation',
    'NSPrivacyCollectedDataTypeSensitiveInfo',
    'NSPrivacyCollectedDataTypeContacts',
    'NSPrivacyCollectedDataTypeEmailsOrTextMessages',
    'NSPrivacyCollectedDataTypePhotosorVideos',
    'NSPrivacyCollectedDataTypeAudioData',
    'NSPrivacyCollectedDataTypeGameplayContent',
    'NSPrivacyCollectedDataTypeCustomerSupport',
    'NSPrivacyCollectedDataTypeOtherUserContent',
    'NSPrivacyCollectedDataTypeBrowsingHistory',
    'NSPrivacyCollectedDataTypeSearchHistory',
    'NSPrivacyCollectedDataTypeUserID',
    'NSPrivacyCollectedDataTypeDeviceID',
    'NSPrivacyCollectedDataTypePurchaseHistory',
    'NSPrivacyCollectedDataTypeProductInteraction',
    'NSPrivacyCollectedDataTypeAdvertisingData',
    'NSPrivacyCollectedDataTypeOtherUsageData',
    'NSPrivacyCollectedDataTypeCrashData',
    'NSPrivacyCollectedDataTypePerformanceData',
    'NSPrivacyCollectedDataTypeOtherDiagnosticData',
    'NSPrivacyCollectedDataTypeEnvironmentScanning',
    'NSPrivacyCollectedDataTypeHands',
    'NSPrivacyCollectedDataTypeHead',
    'NSPrivacyCollectedDataTypeOtherDataTypes',
  };

  static const Set<String> purposes = {
    'NSPrivacyCollectedDataTypePurposeThirdPartyAdvertising',
    'NSPrivacyCollectedDataTypePurposeDeveloperAdvertising',
    'NSPrivacyCollectedDataTypePurposeAnalytics',
    'NSPrivacyCollectedDataTypePurposeProductPersonalization',
    'NSPrivacyCollectedDataTypePurposeAppFunctionality',
    'NSPrivacyCollectedDataTypePurposeOther',
  };

  static const String appFunctionality =
      'NSPrivacyCollectedDataTypePurposeAppFunctionality';
  static const String analytics = 'NSPrivacyCollectedDataTypePurposeAnalytics';
}
