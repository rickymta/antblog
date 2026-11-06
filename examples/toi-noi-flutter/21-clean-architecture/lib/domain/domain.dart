/// Lớp domain của luồng đặt chuyến: yêu cầu, lỗi, cổng repository và use case.
///
/// Chỉ phụ thuộc `toi_noi_core` (entity dùng chung). Không Flutter, không dio,
/// không `toi_noi_api`: `test/architecture_test.dart` kiểm điều này.
library;

export 'quote_request.dart' show QuoteRequest;
export 'request_trip.dart' show RequestTrip;
export 'trip_failure.dart'
    show
        RejectionKind,
        Rejected,
        ServiceFault,
        TripFailure,
        Unreachable,
        UnreachableReason,
        Unreadable;
export 'trip_repository.dart' show TripRepository;
