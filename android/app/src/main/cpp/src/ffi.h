//
// Created by Julia on 28.01.2025.
//

#ifndef ACUBENCH_FFI_H
#define ACUBENCH_FFI_H

#include <cstdint>

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
        TypedU64 sve_mask,
        TypedU64 i8mm_mask
    );

    void drop_bench(void *bench);

    struct CpuConfig {
        size_t crypto_duration;
        size_t crypto_data_len;

        size_t math_duration;
        size_t math_data_len;

        size_t sort_duration;
        size_t sort_data_len;
    };

    struct CpuReport {
        double crypto_tps;
        const char *crypto_err;
        size_t crypto_err_len;

        double math_tps;
        const char *math_err;
        size_t math_err_len;

        double sort_tps;
        const char *sort_err;
        size_t sort_err_len;
    };

    CpuReport* bench_cpu(void *bench, CpuConfig config);
    CpuReport* bench_cpu_multithread(void *bench, CpuConfig config);
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
        const char *alloc_err;
        size_t alloc_err_len;

        double access_seq_avg_t;
        double access_rand_avg_t;
        double access_con_avg_t;
        const char *access_err;
        size_t access_err_len;
    };

    RamReport* bench_ram(void *bench, RamConfig config);
    void drop_ram_report(void *report);

    struct StorageConfig {
        const char *dir;
        size_t dir_len;

        size_t access_seq_iters;
        size_t access_seq_data_len_mb;

        size_t access_rand_iters;
        size_t access_rand_data_len_mb;
    };

    struct StorageReport {
        uint64_t avail_storage;

        double access_seq_avg_t;
        double access_rand_avg_t;
        const char *access_err;
        size_t access_err_len;
    };

    StorageReport* bench_storage(void *bench, StorageConfig config);
    void drop_storage_report(void *report);

    /******** C++ -> Rust ********/

    struct Ops {
        uint64_t ok;
        uint64_t err;
    };

    Ops matrix_mul_sve_i8mm(
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
