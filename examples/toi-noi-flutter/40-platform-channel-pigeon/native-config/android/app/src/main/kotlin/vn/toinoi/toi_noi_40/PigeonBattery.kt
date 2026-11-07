package vn.toinoi.toi_noi_40

import android.util.Log
import io.flutter.plugin.common.BinaryMessenger

/** Cầu nối Pigeon: tên channel, codec và kiểu dữ liệu đến từ `BatteryApi.g.kt`. */
class PigeonBattery(
    private val reader: BatteryReader,
    messenger: BinaryMessenger,
) : BatteryHostApi {
    private val flutterApi = BatteryFlutterApi(messenger)

    init {
        BatteryHostApi.setUp(messenger, this)
    }

    override fun getBatteryStatus(): BatteryStatusMessage =
        try {
            reader.read()
        } catch (e: BatteryUnavailable) {
            throw FlutterError("UNAVAILABLE", e.message, null)
        }

    /** Native gọi Dart. Phải gọi trên main thread. */
    fun push(status: BatteryStatusMessage) {
        flutterApi.onBatteryChanged(status) { result ->
            result.onFailure { Log.w("PigeonBattery", "Dart từ chối onBatteryChanged: $it") }
        }
    }
}
