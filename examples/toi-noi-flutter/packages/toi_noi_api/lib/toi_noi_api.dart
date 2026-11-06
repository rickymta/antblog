/// Client HTTP, DTO và lỗi của Tới Nơi, dùng chung cho app khách và app tài xế.
///
/// Phụ thuộc vào `toi_noi_core` chứ không ngược lại: domain không biết gì về
/// HTTP hay JSON.
library;

export 'src/client.dart' show ToiNoiApiClient;
export 'src/dto.dart' show QuoteDto, QuoteRequestDto, TripDto, TripSnapshot;
export 'src/errors.dart' show ApiException, DtoFormatException;
export 'src/failure.dart'
    show
        ApiFailure,
        ClientFailure,
        NetworkFailure,
        NetworkReason,
        ParseFailure,
        ServerFailure,
        TimeoutFailure,
        TimeoutPhase,
        guardApi,
        toApiFailure;
export 'src/log_interceptor.dart' show SafeLogInterceptor;
export 'src/retry_interceptor.dart' show RetryEvent, RetryInterceptor, Sleep;
