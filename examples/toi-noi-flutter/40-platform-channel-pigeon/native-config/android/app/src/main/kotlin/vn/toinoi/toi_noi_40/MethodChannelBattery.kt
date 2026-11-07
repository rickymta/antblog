package vn.toinoi.toi_noi_40

import io.flutter.plugin.common.BinaryMessenger
import io.flutter.plugin.common.MethodCall
import io.flutter.plugin.common.MethodChannel

/** Cầu nối MethodChannel viết tay: phải khớp `method_channel_battery_gateway.dart`. */
class MethodChannelBattery(
    private val reader: BatteryReader,
    messenger: BinaryMessenger,
) : MethodChannel.MethodCallHandler {
    private val channel = MethodChannel(messenger, CHANNEL)

    init {
        channel.setMethodCallHandler(this)
    }

    // Chạy trên main thread (UI thread), nên đọc nhanh và không chặn.
    override fun onMethodCall(call: MethodCall, result: MethodChannel.Result) {
        when (call.method) {
            "getBatteryStatus" ->
                try {
                    result.success(reader.read().toMap())
                } catch (e: BatteryUnavailable) {
                    result.error("UNAVAILABLE", e.message, null)
                }
            else -> result.notImplemented()
        }
    }

    /** Native gọi Dart. Phải gọi trên main thread. */
    fun push(status: BatteryStatusMessage) {
        channel.invokeMethod("onBatteryChanged", status.toMap())
    }

    companion object {
        const val CHANNEL = "vn.toinoi.driver/battery"
    }
}

internal fun BatteryStatusMessage.toMap(): Map<String, Any> = mapOf(
    "levelPercent" to levelPercent,
    "isCharging" to isCharging,
    "isPowerSaveMode" to isPowerSaveMode,
)
