/// Lớp domain của luồng đặt chuyến (bản rút gọn của lab 21), dùng để thực hành
/// test double. Chỉ phụ thuộc `toi_noi_core`.
library;

export 'booking_log.dart' show BookingLog;
export 'quote_request.dart' show QuoteRequest;
export 'request_trip.dart' show RequestTrip;
export 'trip_failure.dart'
    show Rejected, RejectionKind, ServiceFault, TripFailure, Unreachable;
export 'trip_repository.dart' show TripRepository;
