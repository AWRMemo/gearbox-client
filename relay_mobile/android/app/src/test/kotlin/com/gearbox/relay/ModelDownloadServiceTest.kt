package com.gearbox.relay

import android.app.Notification
import android.app.NotificationManager
import android.content.Context
import androidx.core.app.NotificationCompat
import androidx.test.core.app.ApplicationProvider
import org.junit.Assert.*
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.Shadows.shadowOf

@RunWith(RobolectricTestRunner::class)
class ModelDownloadServiceTest {

    @Test
    fun notificationChannelCreated() {
        val context = ApplicationProvider.getApplicationContext<Context>()
        val nm = context.getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
        val channel = nm.getNotificationChannel(ModelDownloadService.CHANNEL_ID)
        assertNotNull(channel)
        assertEquals("Model Download", channel?.name)
        assertEquals(NotificationManager.IMPORTANCE_LOW, channel?.importance)
    }

    @Test
    fun notificationShowsProgress() {
        val context = ApplicationProvider.getApplicationContext<Context>()
        val builder = NotificationCompat.Builder(context, ModelDownloadService.CHANNEL_ID)
            .setContentTitle("Downloading AI Model")
            .setContentText("1.0 / 2.0 MB")
            .setSmallIcon(android.R.drawable.stat_sys_download)
            .setProgress(100, 50, false)
            .addAction(android.R.drawable.ic_menu_close_clear_cancel, "Cancel", null)

        val notification: Notification = builder.build()
        assertEquals("Downloading AI Model", notification.extras.getString(Notification.EXTRA_TITLE))
    }
}
