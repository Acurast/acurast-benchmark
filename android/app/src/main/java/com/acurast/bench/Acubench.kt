package com.acurast.bench

import kotlin.time.Duration
import kotlin.time.Duration.Companion.seconds

public class Acubench {
    private val ptr: Long

    init {
        ptr = __new__()
    }

    public fun cpu(config: CpuConfig = CpuConfig()): CpuReport {
        val tps = __cpu__(ptr, config.duration.inWholeMilliseconds, config.encodingDataSize, config.mathDataSize, config.sortDataSize) ?: failWithFailedToConstruct("CPU Report")

        return CpuReport(cryptoTps = tps[0], mathTps = tps[1], sortTps = tps[2])
    }

    public fun cpuMultithread(config: CpuConfig = CpuConfig()): CpuReport {
        val tps = __cpu_multithread__(ptr, config.duration.inWholeMilliseconds, config.encodingDataSize, config.mathDataSize, config.sortDataSize) ?: failWithFailedToConstruct("CPU Report")

        return CpuReport(cryptoTps = tps[0], mathTps = tps[1], sortTps = tps[2])
    }

    public fun destroy() {
        __delete__(ptr)
    }

    private fun failWithFailedToConstruct(name: String): Nothing = throw RuntimeException("Failed to construct $name")

    private external fun __new__(): Long
    private external fun __delete__(ptr: Long)

    private external fun __cpu__(ptr: Long, duration: Long, encDataLen: Long, mathDataLen: Long, sortDataLen: Long): DoubleArray?
    private external fun __cpu_multithread__(ptr: Long, duration: Long, encDataLen: Long, mathDataLen: Long, sortDataLen: Long): DoubleArray?

    public data class CpuConfig(
        val duration: Duration = 10.seconds,
        val encodingDataSize: Long = 4096,
        val mathDataSize: Long = 200,
        val sortDataSize: Long = 100_000,
    )

    public data class CpuReport(
        val cryptoTps: Double,
        val mathTps: Double,
        val sortTps: Double,
    )

    public companion object {
        public fun initNative() {
            System.loadLibrary("acubench")
        }
    }
}