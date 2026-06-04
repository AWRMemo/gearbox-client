package com.gearbox.relay

import android.content.Context
import android.content.SharedPreferences
import android.util.Log
import androidx.work.*
import com.google.firebase.messaging.FirebaseMessagingService
import com.google.firebase.messaging.RemoteMessage
import java.util.concurrent.TimeUnit

class FcmService : FirebaseMessagingService() {

    override fun onNewToken(token: String) {
        Log.d(TAG, "New FCM token: $token")
        val prefs: SharedPreferences = getSharedPreferences(
            "FlutterSharedPreferences", Context.MODE_PRIVATE
        )
        prefs.edit()
            .putString("flutter.fcm_token", token)
            .apply()
    }

    override fun onMessageReceived(message: RemoteMessage) {
        Log.d(TAG, "FCM message received: from=${message.from}")

        // Silent push (content-available) triggers background sync
        if (message.data["content-available"] == "1" || message.priority == RemoteMessage.PRIORITY_HIGH) {
            scheduleSyncWork()
        }

        // If notification payload is present, show local notification.
        // Otherwise this is a data-only push — handled silently.
        message.notification?.let {
            scheduleSyncWork()
        }
    }

    private fun scheduleSyncWork() {
        val prefs: SharedPreferences = getSharedPreferences(
            "FlutterSharedPreferences", Context.MODE_PRIVATE
        )
        if (!prefs.contains("flutter.auth_token")) return

        val constraints = Constraints.Builder()
            .setRequiredNetworkType(NetworkType.CONNECTED)
            .build()

        val syncRequest = OneTimeWorkRequestBuilder<SyncWorker>()
            .setConstraints(constraints)
            .setBackoffCriteria(
                BackoffPolicy.EXPONENTIAL,
                WorkRequest.MIN_BACKOFF_MILLIS,
                TimeUnit.MILLISECONDS
            )
            .build()

        WorkManager.getInstance(applicationContext)
            .enqueueUniqueWork(
                SyncWorker.WORK_NAME,
                ExistingWorkPolicy.REPLACE,
                syncRequest
            )
    }

    companion object {
        const val TAG = "RelayFcmService"
    }
}