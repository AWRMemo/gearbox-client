package com.gearbox.relay

import android.content.Intent
import android.os.Bundle
import io.flutter.embedding.android.FlutterActivity

class ShareReceiver : FlutterActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        when (intent?.action) {
            Intent.ACTION_SEND -> {
                if (intent?.type == "text/plain") {
                    val sharedText = intent?.getStringExtra(Intent.EXTRA_TEXT) ?: ""
                    // Pass text to Flutter via method channel
                    flutterEngine?.dartExecutor?.binaryMessenger?.let {
                        val channel = MethodChannelCompat(it, "relay://share")
                        channel.invokeMethod("onShare", sharedText)
                    }
                }
            }
        }
    }

    class MethodChannelCompat(private val messenger: io.flutter.plugin.common.BinaryMessenger, private val name: String) {
        private val channel = io.flutter.plugin.common.MethodChannel(messenger, name)
        fun invokeMethod(method: String, arguments: Any?) {
            channel.invokeMethod(method, arguments)
        }
    }
}
