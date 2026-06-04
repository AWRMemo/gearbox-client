package com.gearbox.relay

import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.Service
import android.content.Context
import android.content.Intent
import android.os.Build
import android.os.IBinder
import androidx.core.app.NotificationCompat
import kotlinx.coroutines.*
import java.io.File
import java.io.FileOutputStream
import java.net.HttpURLConnection
import java.net.URL
import java.security.MessageDigest

/**
 * Foreground service that downloads the on-device SLM from a CDN.
 *
 * Features:
 * – Exponential-backoff retry (immediate → 5 s → 30 s → fail)
 * – SHA-256 hash verification after download
 * – Progress pushed via a static [progressListener] (consumed by [AiPlugin] EventChannel)
 * – Cancellation via ACTION_CANCEL intent
 */
class ModelDownloadService : Service() {

    private val job = SupervisorJob()
    private val scope = CoroutineScope(Dispatchers.IO + job)
    private var currentDownloadJob: Job? = null

    companion object {
        const val EXTRA_URL = "url"
        const val EXTRA_DESTINATION = "destination"
        const val EXTRA_SHA256 = "sha256"
        const val ACTION_CANCEL = "com.gearbox.relay.CANCEL_DOWNLOAD"
        const val CHANNEL_ID = "relay_model_download"
        const val FOREGROUND_ID = 9001

        @Volatile
        var progressListener: ((downloaded: Long, total: Long, status: String) -> Unit)? = null
    }

    override fun onCreate() {
        super.onCreate()
        createNotificationChannel()
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        if (intent?.action == ACTION_CANCEL) {
            currentDownloadJob?.cancel()
            progressListener?.invoke(-1, -1, "cancelled")
            stopForeground(STOP_FOREGROUND_REMOVE)
            stopSelf()
            return START_NOT_STICKY
        }

        val url = intent?.getStringExtra(EXTRA_URL) ?: return START_NOT_STICKY
        val destination = intent.getStringExtra(EXTRA_DESTINATION) ?: return START_NOT_STICKY
        val expectedSha256 = intent.getStringExtra(EXTRA_SHA256) ?: ""

        startForegroundWithProgress(0, "downloading")

        currentDownloadJob = scope.launch {
            val success = downloadWithRetry(url, destination, expectedSha256)
            if (success) {
                progressListener?.invoke(100, 100, "ready")
            } else {
                progressListener?.invoke(-1, -1, "error")
            }
            stopForeground(STOP_FOREGROUND_REMOVE)
            stopSelf()
        }

        return START_NOT_STICKY
    }

    override fun onBind(intent: Intent?): IBinder? = null

    override fun onDestroy() {
        currentDownloadJob?.cancel()
        job.cancel()
        super.onDestroy()
    }

    /**
     * Retry loop with exponential backoff.
     * Returns true only when the file is fully downloaded **and** SHA-256 matches (if supplied).
     */
    private suspend fun downloadWithRetry(
        url: String,
        destination: String,
        expectedSha256: String
    ): Boolean {
        val delaysMs = listOf(0L, 5_000L, 30_000L)
        val destFile = File(destination)
        destFile.parentFile?.mkdirs()

        for ((_, delay) in delaysMs.withIndex()) {
            if (delay > 0) {
                try {
                    delay(delay)
                } catch (_: CancellationException) {
                    destFile.delete()
                    return false
                }
            }

            try {
                val ok = download(url, destFile)
                if (ok && expectedSha256.isNotBlank()) {
                    val hash = sha256Hex(destFile)
                    if (!hash.equals(expectedSha256, ignoreCase = true)) {
                        destFile.delete()
                        continue // hash mismatch → retry
                    }
                }
                return ok
            } catch (e: CancellationException) {
                destFile.delete()
                return false
            } catch (e: Exception) {
                android.util.Log.e("ModelDownloadService", "download attempt failed: ${e.message}")
                // fall through to next retry
            }
        }

        destFile.delete()
        return false
    }

    /** Streams the remote file to disk while reporting progress. */
    private suspend fun download(url: String, destFile: File): Boolean = withContext(Dispatchers.IO) {
        val connection = URL(url).openConnection() as HttpURLConnection
        connection.connectTimeout = 30_000
        connection.readTimeout = 30_000
        connection.instanceFollowRedirects = true
        connection.connect()

        val total = connection.contentLength.toLong()
        if (connection.responseCode !in 200..299) {
            throw IllegalStateException("HTTP ${connection.responseCode}")
        }

        connection.inputStream.use { input ->
            FileOutputStream(destFile).use { output ->
                val buffer = ByteArray(8192)
                var downloaded = 0L
                var read: Int
                while (input.read(buffer).also { read = it } != -1) {
                    if (!isActive) {
                        throw CancellationException()
                    }
                    output.write(buffer, 0, read)
                    downloaded += read
                    if (total > 0) {
                        val pct = ((downloaded * 100) / total).toInt()
                        updateNotification(pct, "downloading")
                        progressListener?.invoke(downloaded, total, "downloading")
                    }
                }
            }
        }
        true
    }

    private fun sha256Hex(file: File): String {
        val digest = MessageDigest.getInstance("SHA-256")
        file.inputStream().use { input ->
            val buffer = ByteArray(8192)
            var read: Int
            while (input.read(buffer).also { read = it } != -1) {
                digest.update(buffer, 0, read)
            }
        }
        return digest.digest().joinToString("") { "%02x".format(it) }
    }

    private fun startForegroundWithProgress(progress: Int, status: String) {
        startForeground(FOREGROUND_ID, buildNotification(progress, status))
    }

    private fun updateNotification(progress: Int, status: String) {
        val nm = getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
        nm.notify(FOREGROUND_ID, buildNotification(progress, status))
    }

    private fun buildNotification(progress: Int, status: String): android.app.Notification {
        val cancelIntent = Intent(this, ModelDownloadService::class.java).apply {
            action = ACTION_CANCEL
        }
        val cancelPending = PendingIntent.getService(
            this,
            0,
            cancelIntent,
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
                PendingIntent.FLAG_IMMUTABLE
            } else {
                PendingIntent.FLAG_UPDATE_CURRENT
            }
        )

        val builder = NotificationCompat.Builder(this, CHANNEL_ID)
            .setContentTitle("Downloading AI Model")
            .setContentText("Status: $status")
            .setSmallIcon(android.R.drawable.stat_sys_download)
            .setOngoing(true)
            .addAction(
                android.R.drawable.ic_menu_close_clear_cancel,
                "Cancel",
                cancelPending
            )

        if (status == "downloading" && progress in 0..100) {
            builder.setProgress(100, progress, false)
        } else {
            builder.setProgress(0, 0, true)
        }
        return builder.build()
    }

    private fun createNotificationChannel() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val channel = NotificationChannel(
                CHANNEL_ID,
                "Model Download",
                NotificationManager.IMPORTANCE_LOW
            ).apply { description = "Shows progress when downloading AI models" }
            val manager = getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
            manager.createNotificationChannel(channel)
        }
    }
}
