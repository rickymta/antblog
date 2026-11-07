/// Công cụ Dart thuần cho bài 56: kiểm Info.plist trước khi lên TestFlight,
/// kế hoạch nhóm tester, thông tin test, hạn build và trạng thái build.
library;

export 'src/beta_checklist.dart'
    show
        AppRole,
        BetaChecklist,
        CheckIssue,
        CheckIssues,
        Severity,
        compareBuilds,
        parseBuild,
        resolveBuildSettings;
export 'src/beta_review.dart'
    show
        BetaSubmission,
        BuildExpiry,
        ReviewExpectation,
        SubmissionDecision,
        checkBetaSubmission,
        expectedReview,
        maxBetaSubmissionsPer24h;
export 'src/build_status.dart' show BuildStatus, StatusColor;
export 'src/plist.dart' show parsePlist;
export 'src/test_information.dart' show InfoIssue, TestInformation;
export 'src/tester_groups.dart'
    show
        AscRole,
        GroupIssue,
        GroupKind,
        LinkMode,
        PublicLink,
        TesterGroup,
        TesterGroups;
export 'src/what_to_test.dart' show buildWhatToTest;
