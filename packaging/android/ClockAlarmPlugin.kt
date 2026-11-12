package com.clockstudy.desktop

import android.Manifest
import android.app.Activity
import android.app.AlarmManager
import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.pm.PackageManager
import android.os.Build
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.Plugin

private const val ALARM_ID = 1
private const val CHANNEL_ID = "timer-completion"

@InvokeArg
class ClockAlarmArgs {
    var deadlineMillis: Long = 0
    var message: String = ""
}

@TauriPlugin
class ClockAlarmPlugin(private val activity: Activity) : Plugin(activity) {
    @Command
    fun schedule(invoke: Invoke) {
        val args = invoke.parseArgs(ClockAlarmArgs::class.java)
        val manager = activity.getSystemService(AlarmManager::class.java)
        val intent = Intent(activity, ClockAlarmReceiver::class.java)
            .putExtra("message", args.message)
        val pending = PendingIntent.getBroadcast(
            activity, ALARM_ID, intent,
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE,
        )
        try {
            manager.setExactAndAllowWhileIdle(
                AlarmManager.ELAPSED_REALTIME_WAKEUP, args.deadlineMillis, pending,
            )
        } catch (error: SecurityException) {
            // If exact alarms were disabled in system settings, keep the notification scheduled.
            manager.setAndAllowWhileIdle(
                AlarmManager.ELAPSED_REALTIME_WAKEUP, args.deadlineMillis, pending,
            )
        }
        invoke.resolve()
    }

    @Command
    fun cancel(invoke: Invoke) {
        val intent = Intent(activity, ClockAlarmReceiver::class.java)
        val pending = PendingIntent.getBroadcast(
            activity, ALARM_ID, intent,
            PendingIntent.FLAG_NO_CREATE or PendingIntent.FLAG_IMMUTABLE,
        )
        if (pending != null) {
            activity.getSystemService(AlarmManager::class.java).cancel(pending)
            pending.cancel()
        }
        invoke.resolve()
    }
}

class ClockAlarmReceiver : BroadcastReceiver() {
    override fun onReceive(context: Context, intent: Intent) {
        if (Build.VERSION.SDK_INT >= 33 &&
            context.checkSelfPermission(Manifest.permission.POST_NOTIFICATIONS) != PackageManager.PERMISSION_GRANTED
        ) return

        val manager = context.getSystemService(NotificationManager::class.java)
        val builder = if (Build.VERSION.SDK_INT >= 26) {
            val channel = NotificationChannel(CHANNEL_ID, "Fim do timer", NotificationManager.IMPORTANCE_HIGH)
            manager.createNotificationChannel(channel)
            Notification.Builder(context, CHANNEL_ID)
        } else {
            @Suppress("DEPRECATION")
            Notification.Builder(context).setDefaults(Notification.DEFAULT_ALL)
                .setPriority(Notification.PRIORITY_HIGH)
        }

        val launch = context.packageManager.getLaunchIntentForPackage(context.packageName)
        if (launch != null) {
            builder.setContentIntent(PendingIntent.getActivity(
                context, ALARM_ID, launch, PendingIntent.FLAG_IMMUTABLE,
            ))
        }
        val notification = builder.setSmallIcon(android.R.drawable.ic_lock_idle_alarm)
            .setContentTitle("Clock Study")
            .setContentText(intent.getStringExtra("message") ?: "Tempo encerrado")
            .setCategory(Notification.CATEGORY_ALARM)
            .setVisibility(Notification.VISIBILITY_PUBLIC)
            .setAutoCancel(true)
            .build()
        manager.notify(ALARM_ID, notification)
    }
}
