#ifndef ACUBENCH_FFI_H
#define ACUBENCH_FFI_H

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif //__cplusplus

    /******** Rust -> C++ ********/

    struct TypedU64 {
        uint8_t t;
        uint64_t v;
    };

    void* new_bench(
        uint64_t total_ram,
        uint64_t avail_storage,
        uint64_t hwcap,
        uint64_t hwcap2,
        struct TypedU64 sve_mask,
        struct TypedU64 i8mm_mask
    );

    void drop_bench(void *bench);

    struct CpuConfig {
        size_t crypto_duration;
        size_t crypto_data_len;

        size_t math_duration;
        size_t math_data_len;
        bool math_simd;

        size_t sort_duration;
        size_t sort_data_len;
    };

    struct CpuReport {
        double crypto_tps;
        double math_tps;
        double sort_tps;
        double score;

        char *err;
    };

    struct CpuReport* bench_cpu(void *bench, struct CpuConfig config);
    struct CpuReport* bench_cpu_multithread(void *bench, struct CpuConfig config);
    void drop_cpu_report(void *report);

    struct RamConfig {
        size_t alloc_iters;
        size_t alloc_data_len;

        size_t access_seq_iters;
        size_t access_seq_data_len;

        size_t access_rand_iters;
        size_t access_rand_data_len;

        size_t access_concurr_iters;
        size_t access_concurr_data_len;
    };

    struct RamReport {
        uint64_t total_mem;
        double alloc_avg_t;
        double access_seq_avg_t;
        double access_rand_avg_t;
        double access_con_avg_t;
        double score;

        char *err;
    };

    struct RamReport* bench_ram(void *bench, struct RamConfig config);
    void drop_ram_report(void *report);

    struct StorageConfig {
        char *dir;

        size_t access_seq_iters;
        size_t access_seq_data_len_mb;

        size_t access_rand_iters;
        size_t access_rand_data_len_mb;
    };

    struct StorageReport {
        uint64_t avail_storage;
        double access_seq_avg_t;
        double access_rand_avg_t;
        double score;

        char *err;
    };

    struct StorageReport* bench_storage(void *bench, struct StorageConfig config);
    void drop_storage_report(void *report);

    /******** C++ -> Rust ********/

    struct Ops {
        uint64_t ok;
        uint64_t err;
    };

    struct Ops matrix_mul_sve_i8mm(
        const int8_t *matrix_a,
        const int8_t *matrix_b /* transposed */,
        int32_t *matrix_r,
        size_t n,
        size_t timeout_timestamp
    );

    struct Ops matrix_mul_naive(
        const int8_t *matrix_a,
        const int8_t *matrix_b,
        int32_t *matrix_r,
        size_t n,
        size_t timeout_timestamp
    );

#ifdef __cplusplus
};
#endif //__cplusplus

#endif //ACUBENCH_FFI_H
