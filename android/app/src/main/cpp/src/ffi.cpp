//
// Created by Julia on 28.01.2025.
//

#include "ffi.h"

#include <cstdint>
#include <ctime>
#include <jni.h>

/******** C++ -> Rust ********/

#ifdef __ARM_FEATURE_SVE
    #include <arm_sve.h>

    int64_t matrix_mul_i8mm(
            const int8_t *matrix_a,
            const int8_t *matrix_b,
            int32_t *matrix_r,
            size_t n,
            size_t timeout_timestamp
    ) {
        int64_t ops = 0;
        for (size_t i = 0; i < n; i++) {
            for (size_t j = 0; j < n; j++) {
                svint32_t sum = svdup_s32(0);
                for (size_t k = 0; k < n; k += svcntb()) {
                    if (timeout_timestamp > 0 && (time(nullptr) * 1000) >= timeout_timestamp) {
                        return -ops;
                    }

                    svbool_t pg = svwhilelt_b8(k, n);

                    svint8_t a = svld1_s8(pg, &matrix_a[i * n + k]);
                    svint8_t b = svld1_s8(pg, &matrix_b[k * n + j]);

                    sum = svdot_s32(sum, a, b);
                    ops += 1;
                }

                matrix_r[i * n + j] = svaddv_s32(svptrue_b32(), sum);
            }
        }

        return ops;
    }
#else
    int64_t matrix_mul_i8mm(
            const int8_t *matrix_a,
            const int8_t *matrix_b,
            int32_t *matrix_r,
            size_t n,
            size_t timeout_timestamp
    ) {
        int64_t ops = 0;
        for (size_t i = 0; i < n; i++) {
            for (size_t j = 0; j < n; j++) {
                int32_t sum = 0;
                for (size_t k = 0; k < n; k++) {
                    if (timeout_timestamp > 0 && (time(nullptr) * 1000) >= timeout_timestamp) {
                        return -ops;
                    }

                    sum += matrix_a[i * n + k] * matrix_b[k * n + j];
                    ops += 1;
                }

                matrix_r[i * n + j] = sum;
            }
        }

        return ops;
    }
#endif //__ARM_FEATURE_SVE
