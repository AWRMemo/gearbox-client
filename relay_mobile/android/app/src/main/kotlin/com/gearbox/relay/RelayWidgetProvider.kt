package com.gearbox.relay

import android.appwidget.AppWidgetManager
import android.appwidget.AppWidgetProvider
import android.content.Context
import android.widget.RemoteViews

class RelayWidgetProvider : AppWidgetProvider() {
    override fun onUpdate(context: Context, appWidgetManager: AppWidgetManager, appWidgetIds: IntArray) {
        for (appWidgetId in appWidgetIds) {
            val views = RemoteViews(context.packageName, R.layout.relay_widget)
            views.setTextViewText(R.id.widget_title, "Relay")
            views.setTextViewText(R.id.widget_capture_count, "0 captures")
            views.setTextViewText(R.id.widget_last_summary, "No captures yet")
            appWidgetManager.updateAppWidget(appWidgetId, views)
        }
    }
}
