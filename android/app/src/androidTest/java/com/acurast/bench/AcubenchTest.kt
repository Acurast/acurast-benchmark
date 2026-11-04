package com.acurast.bench

import android.app.ActivityManager
import android.content.Context
import android.os.StatFs
import androidx.core.content.getSystemService
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.BeforeClass
import org.junit.Test
import org.junit.runner.RunWith
import kotlin.math.abs
import kotlin.time.Duration.Companion.seconds
import kotlin.time.measureTime

/**
 * Instrumented test, which will execute on an Android device.
 *
 * See [testing documentation](http://d.android.com/tools/testing).
 */
@RunWith(AndroidJUnit4::class)
class AcubenchTest {
    companion object {
        /** Free storage moves between two reads; anything beyond this is not drift. */
        private const val STORAGE_TOLERANCE = 64L * 1024 * 1024

        @BeforeClass
        @JvmStatic
        fun setupAll() {
            Acubench.initNative()
        }
    }

    private lateinit var acubench: Acubench
    private lateinit var context: Context

    @Before
    fun setup() {
        context = InstrumentationRegistry.getInstrumentation().targetContext
        acubench = Acubench(context)
    }

    @Test
    fun testAll() {
        val time = measureTime {
            val cpuReport = acubench.cpu()
            assert(cpuReport.cryptoTps > 0)
            assert(cpuReport.mathTps > 0)
            assert(cpuReport.sortTps > 0)
            assert(cpuReport.score == cpuReport.expectedScore)

            val cpuMultithreadReport = acubench.cpuMultithread()
            assert(cpuMultithreadReport.cryptoTps > 0)
            assert(cpuMultithreadReport.mathTps > 0)
            assert(cpuMultithreadReport.sortTps > 0)
            assert(cpuMultithreadReport.score == cpuMultithreadReport.expectedScore)

            val ramReport = acubench.ram()
            assert(ramReport.totalMemory > 0)
            assert(ramReport.allocAvgTime > 0)
            assert(ramReport.accessSequentialAvgTime > 0)
            assert(ramReport.accessRandomAvgTime > 0)
            assert(ramReport.accessConcurrentAvgTime > 0)
            assert(ramReport.score == ramReport.expectedScore)

            val storageReport = acubench.storage(context)
            assert(storageReport.availableStorage > 0)
            assert(storageReport.accessSequentialAvgTime > 0)
            assert(storageReport.accessRandomAvgTime > 0)
            assert(storageReport.score == storageReport.expectedScore)

            println(cpuReport.toPrettyString(descriptor = "Single-core"))
            println(cpuMultithreadReport.toPrettyString(descriptor = "Multicore"))
            println(ramReport.toPrettyString())
            println(storageReport.toPrettyString())
        }

        println("time = $time")
    }

    @Test
    fun testCpu() {
        val duration = 9.seconds
        val time = measureTime {
            val report = acubench.cpu(Acubench.CpuConfig(duration = duration))

            assert(report.cryptoTps > 0)
            assert(report.mathTps > 0)
            assert(report.sortTps > 0)
            assert(report.score == report.expectedScore)

            println(report.toPrettyString(descriptor = "Single-core"))
        }

        assert(time <= duration + 1.seconds)
    }

    @Test
    fun testCpuNoSimd() {
        val duration = 9.seconds
        val time = measureTime {
            val report = acubench.cpu(Acubench.CpuConfig(duration = duration, mathSimd = false))

            assert(report.cryptoTps > 0)
            assert(report.mathTps > 0)
            assert(report.sortTps > 0)
            assert(report.score == report.expectedScore)

            println(report.toPrettyString(descriptor = "Single-core, no SIMD"))
        }

        assert(time <= duration + 1.seconds)
    }

    @Test
    fun testCpuMultithread() {
        val duration = 9.seconds
        val time = measureTime {
            val report = acubench.cpuMultithread(Acubench.CpuConfig(duration = duration))

            assert(report.cryptoTps > 0)
            assert(report.mathTps > 0)
            assert(report.sortTps > 0)
            assert(report.score == report.expectedScore)

            println(report.toPrettyString(descriptor = "Multicore"))
        }

        assert(time <= duration + 1.seconds)
    }

    @Test
    fun testRam() {
        val report = acubench.ram()

        assert(report.totalMemory > 0)
        assert(report.allocAvgTime > 0)
        assert(report.accessSequentialAvgTime > 0)
        assert(report.accessRandomAvgTime > 0)
        assert(report.accessConcurrentAvgTime > 0)
        assert(report.score == report.expectedScore)

        println(report.toPrettyString())
    }

    @Test
    fun testStorage() {
        val report = acubench.storage(context)

        assert(report.availableStorage > 0)
        assert(report.accessSequentialAvgTime > 0)
        assert(report.accessRandomAvgTime > 0)
        assert(report.score == report.expectedScore)

        println(report.toPrettyString())
    }

    @Test
    fun testRamReportsTotalMemoryOfPlatform() {
        val report = acubench.ram(Acubench.RamConfig(iters = 0))

        assertEquals(context.totalMemory, report.totalMemory)
    }

    @Test
    fun testStorageReportsAvailableStorageOfPlatform() {
        val report = acubench.storage(Acubench.StorageConfig(context, iters = 0))

        val drift = abs(report.availableStorage - context.availableStorage)
        assertTrue("available storage drifted by $drift bytes", drift <= STORAGE_TOLERANCE)
    }

    private val Context.totalMemory: Long
        get() = ActivityManager.MemoryInfo()
            .also { getSystemService<ActivityManager>()!!.getMemoryInfo(it) }
            .totalMem

    private val Context.availableStorage: Long
        get() = StatFs(dataDir.path).availableBytes

    private val Acubench.CpuReport.expectedScore: Double
        get() = (cryptoTps + mathTps + sortTps) / 3.0

    private val Acubench.RamReport.expectedScore: Double
        get() = (allocAvgTime.inv() + accessSequentialAvgTime.inv() + accessRandomAvgTime.inv() + accessConcurrentAvgTime.inv()) / 4.0

    private val Acubench.StorageReport.expectedScore: Double
        get() = (accessSequentialAvgTime.inv() + accessRandomAvgTime.inv()) / 2.0

    private fun Double.inv(): Double = if (this == 0.0) 0.0 else 1.0 / this
}