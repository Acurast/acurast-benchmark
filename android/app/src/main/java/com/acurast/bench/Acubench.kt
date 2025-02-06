package com.acurast.bench

import android.app.ActivityManager
import android.content.Context
import android.os.StatFs
import androidx.core.content.getSystemService
import java.io.File
import kotlin.time.Duration
import kotlin.time.Duration.Companion.seconds

public class Acubench(context: Context) {
    private val ptr: Long

    init {
        ptr = __new__(context.totalRam ?: 0, context.availableStorage)
    }

    public fun cpu(config: CpuConfig = CpuConfig()): CpuReport =
        __cpu__(ptr, config.duration.inWholeMilliseconds, config.encodingDataSize, config.mathDataSize, config.sortDataSize)

    public fun cpuMultithread(config: CpuConfig = CpuConfig()): CpuReport =
        __cpu_multithread__(ptr, config.duration.inWholeMilliseconds, config.encodingDataSize, config.mathDataSize, config.sortDataSize)

    public fun ram(config: RamConfig = RamConfig()): RamReport =
        __ram__(ptr, config.allocDataSize, config.accessDataSize, config.iters)

    public fun storage(config: StorageConfig): StorageReport =
        __storage__(ptr, config.dir.absolutePath.toByteArray(charset = Charsets.UTF_8), config.accessDataSizeMB, config.iters)

    public fun storage(context: Context): StorageReport =
        storage(StorageConfig(context))

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

    private val Context.availableStorage: Long
        get() = StatFs(dataDir.path).availableBytes

    private external fun __new__(totalRam: Long, availStorage: Long): Long
    private external fun __delete__(ptr: Long)

    private external fun __cpu__(ptr: Long, duration: Long, encDataLen: Long, mathDataLen: Long, sortDataLen: Long): CpuReport
    private external fun __cpu_multithread__(ptr: Long, duration: Long, encDataLen: Long, mathDataLen: Long, sortDataLen: Long): CpuReport

    private external fun __ram__(ptr: Long, allocDataLen: Long, accessDataLen: Long, iters: Long): RamReport

    private external fun __storage__(ptr: Long, dir: ByteArray, accessDataLenMB: Long, iters: Long): StorageReport

    public data class CpuConfig(
        val duration: Duration = DURATION_DEFAULT,
        val encodingDataSize: Long = ENCODING_DATA_SIZE_DEFAULT,
        val mathDataSize: Long = MATH_DATA_SIZE_DEFAULT,
        val sortDataSize: Long = SORT_DATA_SIZE_DEFAULT,
    ) {
        public companion object {
            private val DURATION_DEFAULT = 3.seconds
            private const val ENCODING_DATA_SIZE_DEFAULT = 10 * KB
            private const val MATH_DATA_SIZE_DEFAULT = 200L
            private const val SORT_DATA_SIZE_DEFAULT = 100_000L
        }
    }

    public data class CpuReport(
        val cryptoTps: Double,
        val mathTps: Double,
        val sortTps: Double,
    ) {
        public companion object
    }

    public data class RamConfig(
        val allocDataSize: Long = ALLOC_DATA_SIZE_DEFAULT,
        val accessDataSize: Long = ACCESS_DATA_SIZE_DEFAULT,
        val iters: Long = ITERS_DEFAULT,
    ) {
        public companion object {
            private const val ALLOC_DATA_SIZE_DEFAULT = 64 * MB
            private const val ACCESS_DATA_SIZE_DEFAULT = 64 * KB
            private const val ITERS_DEFAULT = 10L
        }
    }

    public data class RamReport(
        val totalMemory: Long,
        val allocAvgTime: Double,
        val accessSequentialAvgTime: Double,
        val accessRandomAvgTime: Double,
        val accessConcurrentAvgTime: Double,
    ) {
        public companion object
    }

    public data class StorageConfig(
        val dir: File,
        val accessDataSizeMB: Long = ACCESS_DATA_SIZE_MB_DEFAULT,
        val iters: Long = ITERS_DEFAULT,
    ) {
        public constructor(
            context: Context,
            accessDataSizeMB: Long = ACCESS_DATA_SIZE_MB_DEFAULT,
            iters: Long = ITERS_DEFAULT,
        ) : this(context.cacheDir, accessDataSizeMB, iters)

        public companion object {
            private const val ACCESS_DATA_SIZE_MB_DEFAULT = 50L
            private const val ITERS_DEFAULT = 1L
        }
    }

    public data class StorageReport(
        val availableStorage: Long,
        val accessSequentialAvgTime: Double,
        val accessRandomAvgTime: Double,
    ) {
        public companion object
    }

    public companion object {
        private const val KB = 1024L
        private const val MB = KB * KB

        public fun initNative() {
            System.loadLibrary("acubench")
        }
    }
}