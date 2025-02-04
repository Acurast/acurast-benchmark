package com.acurast.bench

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

    @Before
    fun setup() {
        val appContext = InstrumentationRegistry.getInstrumentation().targetContext
        acubench = Acubench(appContext)
    }

    @Test
    fun testCpu() {
        val duration = 9.seconds
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

        assert(report.totalMem > 0)
        assert(report.allocAvgTime > 0)
        assert(report.accessSequentialAvgTime > 0)
        assert(report.accessRandomAvgTime > 0)
        assert(report.accessConcurrentAvgTime > 0)

        println("$report")
    }
}