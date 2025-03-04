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
        __cpu__(
            ptr,
            config.cryptoDuration.inWholeMilliseconds,
            config.cryptoDataSize,
            config.mathDuration.inWholeMilliseconds,
            config.mathDataSize,
            config.sortDuration.inWholeMilliseconds,
            config.sortDataSize,
        )

    public fun cpuMultithread(config: CpuConfig = CpuConfig()): CpuReport =
        __cpu_multithread__(
            ptr,
            config.cryptoDuration.inWholeMilliseconds,
            config.cryptoDataSize,
            config.mathDuration.inWholeMilliseconds,
            config.mathDataSize,
            config.sortDuration.inWholeMilliseconds,
            config.sortDataSize,
        )

    public fun ram(config: RamConfig = RamConfig()): RamReport =
        __ram__(
            ptr,
            config.allocIters,
            config.allocDataSize,
            config.accessSequentialIters,
            config.accessSequentialDataSize,
            config.accessRandomIters,
            config.accessRandomDataSize,
            config.accessConcurrentIters,
            config.accessConcurrentDataSize,
        )

    public fun storage(config: StorageConfig): StorageReport =
        __storage__(
            ptr,
            config.dir.absolutePath.toByteArray(charset = Charsets.UTF_8),
            config.accessSequentialIters,
            config.accessSequentialDataSizeMB,
            config.accessRandomIters,
            config.accessRandomDataSizeMB,
        )

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

    private external fun __cpu__(
        ptr: Long,
        cryptoDuration: Long,
        cryptoDataLen: Long,
        mathDuration: Long,
        mathDataLen: Long,
        sortDuration: Long,
        sortDataLen: Long,
    ): CpuReport
    private external fun __cpu_multithread__(
        ptr: Long,
        cryptoDuration: Long,
        cryptoDataLen: Long,
        mathDuration: Long,
        mathDataLen: Long,
        sortDuration: Long,
        sortDataLen: Long,
    ): CpuReport

    private external fun __ram__(
        ptr: Long,
        allocIters: Long,
        allocDataLen: Long,
        accessSeqIters: Long,
        accessSeqDataLen: Long,
        accessRandIters: Long,
        accessRandDataLen: Long,
        accessConcurrIters: Long,
        accessConcurrDataLen: Long,
    ): RamReport

    private external fun __storage__(
        ptr: Long,
        dir: ByteArray,
        accessSeqIters: Long,
        accessSeqDataLenMB: Long,
        accessRandIters: Long,
        accessRandDataLenMB: Long,
    ): StorageReport

    public data class CpuConfig(
        val cryptoDuration: Duration = DURATION_DEFAULT,
        val cryptoDataSize: Long = CRYPTO_DATA_SIZE_DEFAULT,
        val mathDuration: Duration = DURATION_DEFAULT,
        val mathDataSize: Long = MATH_DATA_SIZE_DEFAULT,
        val sortDuration: Duration = DURATION_DEFAULT,
        val sortDataSize: Long = SORT_DATA_SIZE_DEFAULT,
    ) {
        public constructor(
            duration: Duration = DURATION_DEFAULT,
            cryptoDataSize: Long = CRYPTO_DATA_SIZE_DEFAULT,
            mathDataSize: Long = MATH_DATA_SIZE_DEFAULT,
            sortDataSize: Long = SORT_DATA_SIZE_DEFAULT,
        ) : this(duration, cryptoDataSize, duration, mathDataSize, duration, sortDataSize)

        public companion object {
            private val DURATION_DEFAULT = 1.seconds
            private const val CRYPTO_DATA_SIZE_DEFAULT = 10 * KB
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
        val allocIters: Long = ITERS_DEFAULT,
        val allocDataSize: Long = ALLOC_DATA_SIZE_DEFAULT,
        val accessSequentialIters: Long = ITERS_DEFAULT,
        val accessSequentialDataSize: Long = ACCESS_DATA_SIZE_DEFAULT,
        val accessRandomIters: Long = ITERS_DEFAULT,
        val accessRandomDataSize: Long = ACCESS_DATA_SIZE_DEFAULT,
        val accessConcurrentIters: Long = ITERS_DEFAULT,
        val accessConcurrentDataSize: Long = ACCESS_DATA_SIZE_DEFAULT,
    ) {
        public constructor(
            allocIters: Long = ITERS_DEFAULT,
            allocDataSize: Long = ALLOC_DATA_SIZE_DEFAULT,
            accessIters: Long = ITERS_DEFAULT,
            accessDataSize: Long = ACCESS_DATA_SIZE_DEFAULT,
        ) : this(
            allocIters,
            allocDataSize,
            accessIters,
            accessDataSize,
            accessIters,
            accessDataSize,
            accessIters,
            accessDataSize,
        )

        public constructor(
            iters: Long = ITERS_DEFAULT,
            allocDataSize: Long = ALLOC_DATA_SIZE_DEFAULT,
            accessDataSize: Long = ACCESS_DATA_SIZE_DEFAULT,
        ) : this(
            iters,
            allocDataSize,
            iters,
            accessDataSize,
            iters,
            accessDataSize,
            iters,
            accessDataSize
        )

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
        val accessSequentialIters: Long = ITERS_DEFAULT,
        val accessSequentialDataSizeMB: Long = ACCESS_DATA_SIZE_MB_DEFAULT,
        val accessRandomIters: Long = ITERS_DEFAULT,
        val accessRandomDataSizeMB: Long = ACCESS_DATA_SIZE_MB_DEFAULT,
    ) {
        public constructor(
            context: Context,
            accessSequentialIters: Long = ITERS_DEFAULT,
            accessSequentialDataSizeMB: Long = ACCESS_DATA_SIZE_MB_DEFAULT,
            accessRandomIters: Long = ITERS_DEFAULT,
            accessRandomDataSizeMB: Long = ACCESS_DATA_SIZE_MB_DEFAULT,
        ) : this(context.cacheDir, accessSequentialIters, accessSequentialDataSizeMB, accessRandomIters, accessRandomDataSizeMB)

        public constructor(
            dir: File,
            iters: Long = ITERS_DEFAULT,
            accessDataSizeMB: Long = ACCESS_DATA_SIZE_MB_DEFAULT,
        ) : this(dir, iters, accessDataSizeMB, iters, accessDataSizeMB)

        public constructor(
            context: Context,
            iters: Long = ITERS_DEFAULT,
            accessDataSizeMB: Long = ACCESS_DATA_SIZE_MB_DEFAULT,
        ) : this(context, iters, accessDataSizeMB, iters, accessDataSizeMB)

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