package com.acurast.bench

import android.app.ActivityManager
import android.content.Context
import androidx.core.content.getSystemService
import kotlin.time.Duration
import kotlin.time.Duration.Companion.seconds

public class Acubench(context: Context) {
    private val ptr: Long

    init {
        ptr = __new__(context.totalRam ?: 0)
    }

    public fun cpu(config: CpuConfig = CpuConfig()): CpuReport =
        __cpu__(ptr, config.duration.inWholeMilliseconds, config.encodingDataSize, config.mathDataSize, config.sortDataSize)

    public fun cpuMultithread(config: CpuConfig = CpuConfig()): CpuReport =
        __cpu_multithread__(ptr, config.duration.inWholeMilliseconds, config.encodingDataSize, config.mathDataSize, config.sortDataSize)

    public fun ram(config: RamConfig = RamConfig()): RamReport =
        __ram__(ptr, config.allocDataSize, config.accessDataSize, config.iters)

    public fun destroy() {
        __delete__(ptr)
    }

    private val Context.totalRam: Long?
        get() {
            val activityManager = getSystemService<ActivityManager>() ?: return null
            val memInfo = ActivityManager.MemoryInfo().also {
                activityManager.getMemoryInfo(it)
            }

            return memInfo.totalMem
        }

    private external fun __new__(totalRam: Long): Long
    private external fun __delete__(ptr: Long)

    private external fun __cpu__(ptr: Long, duration: Long, encDataLen: Long, mathDataLen: Long, sortDataLen: Long): CpuReport
    private external fun __cpu_multithread__(ptr: Long, duration: Long, encDataLen: Long, mathDataLen: Long, sortDataLen: Long): CpuReport

    private external fun __ram__(ptr: Long, allocDataLen: Long, accessDataLen: Long, iters: Long): RamReport

    public data class CpuConfig(
        val duration: Duration = 10.seconds,
        val encodingDataSize: Long = 4096,
        val mathDataSize: Long = 200,
        val sortDataSize: Long = 100_000,
    ) {
        public companion object
    }

    public data class CpuReport(
        val cryptoTps: Double,
        val mathTps: Double,
        val sortTps: Double,
    ) {
        public companion object
    }

    public data class RamConfig(
        val allocDataSize: Long = 64 * 1024 * 1024,
        val accessDataSize: Long = 64 * 1024,
        val iters: Long = 100,
    ) {
        public companion object
    }

    public data class RamReport(
        val totalMem: Long,
        val allocAvgTime: Double,
        val accessSequentialAvgTime: Double,
        val accessRandomAvgTime: Double,
        val accessConcurrentAvgTime: Double,
    ) {
        public companion object
    }

    public companion object {
        public fun initNative() {
            System.loadLibrary("acubench")
        }
    }
}