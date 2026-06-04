package com.gearbox.relay

import io.flutter.embedding.android.FlutterActivity
import io.flutter.embedding.engine.FlutterEngine
import android.os.Bundle

class MainActivity : FlutterActivity() {
    private var aiPlugin: AiPlugin? = null
    private var modelDownloadPlugin: ModelDownloadPlugin? = null

    override fun configureFlutterEngine(flutterEngine: FlutterEngine) {
        super.configureFlutterEngine(flutterEngine)
        aiPlugin = AiPlugin(applicationContext).apply {
            registerWith(flutterEngine)
        }
        modelDownloadPlugin = ModelDownloadPlugin(applicationContext).apply {
            registerWith(flutterEngine)
        }
    }

    override fun cleanUpFlutterEngine(flutterEngine: FlutterEngine) {
        super.cleanUpFlutterEngine(flutterEngine)
        aiPlugin?.onDetachedFromEngine()
        aiPlugin = null
        modelDownloadPlugin = null
    }
}
