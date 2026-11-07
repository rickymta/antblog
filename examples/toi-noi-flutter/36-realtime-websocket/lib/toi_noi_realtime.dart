/// Client Realtime Gateway của Tới Nơi (bài 36): kết nối WebSocket có
/// reconnect, heartbeat, resume theo `sequence` và gộp vị trí tài xế.
library;

export 'backoff.dart' show Backoff;
export 'realtime_client.dart'
    show
        CloseCause,
        RealtimeClient,
        RealtimeIssue,
        RealtimeState,
        RealtimeStatus;
export 'realtime_lifecycle.dart' show RealtimeLifecycleBinding;
export 'realtime_protocol.dart'
    show
        DriverLocation,
        DriverOffer,
        TripEventEnvelope,
        closeCodeUnauthorized,
        parseServerFrame;
export 'realtime_transport.dart'
    show
        RealtimeAuthRejected,
        RealtimeConnection,
        RealtimeTransport,
        WebSocketTransport;
