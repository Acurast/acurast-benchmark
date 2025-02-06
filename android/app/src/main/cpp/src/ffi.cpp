//
// Created by Julia on 28.01.2025.
//

#include "ffi.h"

#include <cstdint>
#include <ctime>
#include <jni.h>


/******** C++ -> Rust ********/

#ifdef __aarch64__
    __attribute__((target("arch=armv8-a+sve")))
    #include <arm_sve.h>

    __attribute__((target("arch=armv8-a+sve")))
    Ops matrix_mul_sve_i8mm(
            const int8_t *matrix_a,
            const int8_t *matrix_b /* transposed */,
            int32_t *matrix_r,
            size_t n,
            size_t timeout_timestamp
    ) {
        uint64_t ops = 0;

        uint64_t vl = svcntb();
        int8_t col_b[vl];
        for (size_t i = 0; i < n; i++) {
            for (size_t j = 0; j < n; j++) {
                svint32_t sum = svdup_s32(0);
                for (size_t k = 0; k < n; k += vl) {
                    if (timeout_timestamp > 0 && (time(nullptr) * 1000) >= timeout_timestamp) {
                        return Ops {.ok = 0, .err = ops};
                    }

                    svbool_t pg = svwhilelt_b8(k, n);

                    svint8_t a = svld1_s8(pg, &matrix_a[i * n + k]);
                    svint8_t b = svld1_s8(pg, &matrix_b[j * n + k]);

                    sum = svdot_s32(sum, a, b);
                }

                matrix_r[i * n + j] = svaddv_s32(svptrue_b32(), sum);
                ops += n;
            }
        }

        return Ops {.ok = ops, .err = 0};
    }
#else
Ops matrix_mul_sve_i8mm(
            const int8_t *matrix_a,
            const int8_t *matrix_b /* transposed */,
            int32_t *matrix_r,
            size_t n,
            size_t timeout_timestamp
    ) {
        return Ops {.ok = 0, .err = 0};
    }
#endif //__aarch64__