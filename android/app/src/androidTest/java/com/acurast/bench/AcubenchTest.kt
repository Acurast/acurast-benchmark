package com.acurast.bench

import android.content.Context
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import org.junit.Before
import org.junit.BeforeClass
import org.junit.Test
import org.junit.runner.RunWith
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

            val cpuMultithreadReport = acubench.cpuMultithread()
            assert(cpuMultithreadReport.cryptoTps > 0)
            assert(cpuMultithreadReport.mathTps > 0)
            assert(cpuMultithreadReport.sortTps > 0)

            val ramReport = acubench.ram()
            assert(ramReport.totalMemory > 0)
            assert(ramReport.allocAvgTime > 0)
            assert(ramReport.accessSequentialAvgTime > 0)
            assert(ramReport.accessRandomAvgTime > 0)
            assert(ramReport.accessConcurrentAvgTime > 0)

            val storageReport = acubench.storage(context)
            assert(storageReport.availableStorage > 0)
            assert(storageReport.accessSequentialAvgTime > 0)
            assert(storageReport.accessRandomAvgTime > 0)

            println("cpu (singlecore) $cpuReport")
            println("cpu (multicore) $cpuMultithreadReport")
            println("ram $ramReport")
            println("storage $storageReport")
        }

        println("time = $time")
    }

    @Test
    fun testCpu() {
        val duration = 9000.seconds
        val time = measureTime {
            val report = acubench.cpu(Acubench.CpuConfig(duration = duration))

            assert(report.cryptoTps > 0)
            assert(report.mathTps > 0)
            assert(report.sortTps > 0)
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
    }

    @Test
    fun testStorage() {
        val report = acubench.storage(context)

        assert(report.availableStorage > 0)
        assert(report.accessSequentialAvgTime > 0)
        assert(report.accessRandomAvgTime > 0)
    }
}