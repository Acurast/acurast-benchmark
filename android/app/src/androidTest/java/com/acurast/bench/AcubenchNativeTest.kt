package com.acurast.bench

import androidx.test.ext.junit.runners.AndroidJUnit4
import org.junit.BeforeClass
import org.junit.Test
import org.junit.runner.RunWith

/**
 * Instrumented test, which will execute on an Android device.
 *
 * See [testing documentation](http://d.android.com/tools/testing).
 */
@RunWith(AndroidJUnit4::class)
class AcubenchNativeTest {
    companion object {
        @BeforeClass
        @JvmStatic
        fun setupAll() {
            System.loadLibrary("acubenchtest")
        }
    }

    @Test
    fun testMatrixMulI8mm() {
        assert(__test_matrix_mul_i8mm__())
    }

    private external fun __test_matrix_mul_i8mm__(): Boolean
}