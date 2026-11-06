/// Kiểm tra gói phát hành App Store của Tới Nơi bằng Dart thuần: metadata cho
/// `fastlane deliver`, ảnh chụp, privacy manifest, privacy label, và kế hoạch
/// phased release.
library;

export 'src/loader.dart' show loadReleaseInput, loadScreenshots;
export 'src/metadata.dart'
    show AppMetadata, LocalizedMetadata, ReviewInformation;
export 'src/phased_release.dart'
    show PhasedReleasePlan, RolloutAction, RolloutMetrics, RolloutThresholds;
export 'src/plist.dart' show parsePlist;
export 'src/privacy_manifest.dart'
    show
        AccessedApi,
        CollectedData,
        InfoPlist,
        LabelEntry,
        PrivacyLabel,
        PrivacyManifest;
export 'src/release_checklist.dart'
    show
        Finding,
        ReleaseChecklist,
        ReleaseInput,
        Severity,
        StoreApp,
        checkScreenshots,
        splitByDevice;
export 'src/screenshot.dart' show PngInfo;
export 'src/store_limits.dart'
    show ManifestVocabulary, RequiredReasonApi, ScreenshotSizes, StoreLimits;
