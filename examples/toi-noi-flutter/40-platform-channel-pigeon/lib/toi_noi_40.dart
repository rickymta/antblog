/// API công khai của lab bài 40.
library;

export 'src/battery_event_stream.dart'
    show batteryEvents, batteryEventsChannelName;
export 'src/battery_gateway.dart'
    show BatteryGateway, BatteryUnavailableException;
export 'src/battery_status.dart' show BatteryStatus;
export 'src/fare_rounder.dart'
    show DartFareRounder, FareRounder, FfiFareRounder;
export 'src/method_channel_battery_gateway.dart'
    show
        MethodChannelBatteryGateway,
        batteryChannelName,
        getBatteryStatusMethod,
        onBatteryChangedMethod;
export 'src/pigeon_battery_gateway.dart' show PigeonBatteryGateway;
export 'src/report_interval.dart'
    show ReportInterval, locationReportInterval, watchReportInterval;
