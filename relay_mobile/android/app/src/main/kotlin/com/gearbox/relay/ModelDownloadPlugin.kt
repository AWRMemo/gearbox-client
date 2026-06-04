package com.gearbox.relay

import android.content.ComponentName
import android.content.Context
import android.content.Intent
import android.content.ServiceConnection
import android.os.IBinder
import io.flutter.embedding.engine.FlutterEngine
import io.flutter.plugin.common.EventChannel
import io.flutter.plugin.common.MethodCall
import io.flutter.plugin.common.MethodChannel
import io.flutter.plugin.common.MethodChannel.MethodCallHandler
import io.flutter.plugin.common.MethodChannel.Result

/**
 * Flutter plugin that bridges to [ModelDownloadService] for reliable background
 * model downloads on Android.
 */
class ModelDownloadPlugin(private val context: Context) : MethodCallHandler, EventChannel.StreamHandler {

    private var channel: MethodChannel? = null
    private var eventSink: EventChannel.EventSink? = null

    fun registerWith(flutterEngine: FlutterEngine) {
        channel = MethodChannel(flutterEngine.dartExecutor.binaryMessenger, CHANNEL_NAME)
        channel?.setMethodCallHandler(this)
        val eventChannel = EventChannel(flutterEngine.dartExecutor.binaryMessenger, "$CHANNEL_NAME/events")
        eventChannel.setStreamHandler(this)
    }

    override fun onMethodCall(call: MethodCall, result: Result) {
        when (call.method) {
            "startDownload" -> {
                val url = call.argument<String>("url")
                val dest = call.argument<String>("savePath")
                val sha256 = call.argument<String>("sha256")
                if (url == null || dest == null) {
                    result.error("INVALID_ARGUMENT", "Missing url or savePath", null)
                    return
                }
                val intent = Intent(context, ModelDownloadService::class.java).apply {
                    putExtra(ModelDownloadService.EXTRA_URL, url)
                    putExtra(ModelDownloadService.EXTRA_DESTINATION, dest)
                    putExtra(ModelDownloadService.EXTRA_SHA256, sha256 ?: "")
                }
                context.startForegroundService(intent)
                result.success(true)
            }
            "cancelDownload" -> {
                val cancelIntent = Intent(context, ModelDownloadService::class.java).apply {
                    action = ModelDownloadService.ACTION_CANCEL
                }
                context.startService(cancelIntent)
                result.success(true)
            }
            else -> result.notImplemented()
        }
    }

    override fun onListen(arguments: Any?, events: EventChannel.EventSink?) {
        eventSink = events
    }

    override fun onCancel(arguments: Any?) {
        eventSink = null
    }

    companion object {
        const val CHANNEL_NAME = "com.gearbox.model_download"
    }
}
