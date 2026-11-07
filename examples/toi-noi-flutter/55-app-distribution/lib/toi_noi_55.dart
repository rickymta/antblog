/// Lab bài 55: phân phối nội bộ qua Firebase App Distribution.
///
/// - [ReleaseNotes]: release notes tiếng Việt sinh từ danh sách commit.
/// - [DistributionConfig]: kiểm tra `app-distribution.json` và dựng tham số
///   cho `firebase appdistribution:distribute`.
library;

export 'src/cli.dart' show CliResult, UsageError, buildParser, runCli;
export 'src/distribution_config.dart'
    show
        DistributionConfig,
        DistributionException,
        DistributionPlatform,
        DistributionTarget;
export 'src/release_notes.dart'
    show
        Commit,
        NoteItem,
        NoteSection,
        ReleaseNotes,
        classifyCommit,
        defaultMaxNotesLength,
        minNotesLength,
        truncateAtWord;
