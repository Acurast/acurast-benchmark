//
// Created by Julia on 29.01.2025.
//

#include <jni.h>

#include "../src/ffi.h"

extern "C"
JNIEXPORT jboolean JNICALL
Java_com_acurast_bench_AcubenchNativeTest__1_1test_1matrix_1mul_1i8mm_1_1(JNIEnv *env,
                                                                          jobject thiz) {

    const int8_t matrix_a[] = {
            80, 43, 16, 5,
            70, 41, 38, 62,
            31, 19, 97, 39,
            66, 6, 40, 28
    };

    const int8_t matrix_b[] = {
            24, 12, 24, 29,
            83, 59, 32, 44,
            97, 38, 67, 13,
            98, 64, 68, 29,
    };

    int32_t matrix_r[16] = {0};
    const int32_t matrix_r_expected[] = {
            7531, 4425, 4708, 4565,
            14845, 8671, 9754, 6126,
            15552, 7675, 10503, 4127,
            8706, 4458, 6360, 3510,
    };

    int64_t ops_expected = 64;

    int64_t ops = matrix_mul_i8mm(matrix_a, matrix_b, matrix_r, 4, 0);
    if (ops != ops_expected) {
        return false;
    }

    bool equal = true;
    for (auto i = 0; i < 16; i++) {
        if (matrix_r_expected[i] != matrix_r[i]) {
            equal = false;
            break;
        }
    }

    return equal;
}