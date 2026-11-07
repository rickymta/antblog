package vn.toinoi.toi_noi_40

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.IntentFilter
import android.os.PowerManager
import androidx.core.content.ContextCompat
import io.flutter.embedding.android.FlutterActivity
import io.flutter.embedding.engine.FlutterEngine

class MainActivity : FlutterActivity() {
    private var receiver: BroadcastReceiver? = null

    override fun configureFlutterEngine(flutterEngine: FlutterEngine) {
        super.configureFlutterEngine(flutterEngine)
        val messenger = flutterEngine.dartExecutor.binaryMessenger
        val reader = BatteryReader(applicationContext)
        // Lab nối cả hai cầu để so sánh; app thật chọn một.
        val channelBridge = MethodChannelBattery(reader, messenger)
        val pigeonBridge = PigeonBattery(reader, messenger)

        val filter = IntentFilter(Intent.ACTION_BATTERY_CHANGED).apply {
            addAction(PowerManager.ACTION_POWER_SAVE_MODE_CHANGED)
        }
        // Không truyền Handler nên onReceive chạy trên main thread: đúng luồng
        // để gọi channel về Dart.
        val newReceiver = object : BroadcastReceiver() {
            override fun onReceive(context: Context, intent: Intent) {
                try {
                    val status = reader.read()
                    channelBridge.push(status)
                    pigeonBridge.push(status)
                } catch (_: BatteryUnavailable) {
                    // Không có gì để báo.
                }
            }
        }
        ContextCompat.registerReceiver(
            this, newReceiver, filter, ContextCompat.RECEIVER_NOT_EXPORTED,
        )
        receiver = newReceiver
    }

    override fun cleanUpFlutterEngine(flutterEngine: FlutterEngine) {
        receiver?.let { unregisterReceiver(it) }
        receiver = null
        super.cleanUpFlutterEngine(flutterEngine)
    }
}
