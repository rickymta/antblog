package vn.toinoi.toi_noi_40

import android.content.Context
import android.os.BatteryManager
import android.os.PowerManager

/** Không đọc được pin (máy không có pin, hoặc hệ điều hành không báo). */
class BatteryUnavailable(message: String) : Exception(message)

/** Đọc pin từ dịch vụ hệ thống; không cần quyền nào trong AndroidManifest. */
class BatteryReader(private val context: Context) {
    fun read(): BatteryStatusMessage {
        val battery = context.getSystemService(Context.BATTERY_SERVICE) as BatteryManager
        // Trả Integer.MIN_VALUE khi máy không hỗ trợ thuộc tính này.
        val level = battery.getIntProperty(BatteryManager.BATTERY_PROPERTY_CAPACITY)
        if (level < 0 || level > 100) {
            throw BatteryUnavailable("Battery level not available.")
        }
        val power = context.getSystemService(Context.POWER_SERVICE) as PowerManager
        return BatteryStatusMessage(
            levelPercent = level.toLong(),
            isCharging = battery.isCharging,
            isPowerSaveMode = power.isPowerSaveMode,
        )
    }
}
