/// Thư viện domain dùng chung của Tới Nơi: tiền, cước, trạng thái và sự kiện
/// của chuyến.
///
/// Chỉ những gì được `export` ở đây mới là API công khai. Mọi thứ trong
/// `lib/src/` là chi tiết nội bộ, đổi lúc nào cũng được.
library;

export 'src/fare.dart' show FarePolicy, calculateFare, calculateFareVnd;
export 'src/money.dart' show Money;
export 'src/service_kind.dart' show ServiceKind;
export 'src/trip.dart' show Trip;
export 'src/trip_event.dart'
    show
        Cancelled,
        CancelledBy,
        Completed,
        DriverAccepted,
        DriverArrived,
        PickedUp,
        SearchStarted,
        TripEvent,
        TripStarted;
export 'src/trip_queries.dart' show TripIterableX;
export 'src/trip_status.dart' show TripStatus;
