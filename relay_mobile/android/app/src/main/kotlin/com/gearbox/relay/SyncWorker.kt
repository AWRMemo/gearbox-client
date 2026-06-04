package com.gearbox.relay

import android.content.Context
import androidx.work.CoroutineWorker
import androidx.work.WorkerParameters
import android.content.SharedPreferences

class SyncWorker(context: Context, params: WorkerParameters) : CoroutineWorker(context, params) {
    override suspend fun doWork(): Result {
        return try {
            val prefs: SharedPreferences = applicationContext.getSharedPreferences(
                "FlutterSharedPreferences", Context.MODE_PRIVATE
            )

            val hasAuth = prefs.contains("flutter.auth_token")
            if (hasAuth) {
                // Set flag so the Dart service picks up the sync on next foreground launch.
                prefs.edit()
                    .putBoolean("flutter.needs_background_sync", true)
                    .apply()
            }

            Result.success()
        } catch (e: Exception) {
            Result.retry()
        }
    }

    companion object {
        const val WORK_NAME = "relay_background_sync"
    }
}