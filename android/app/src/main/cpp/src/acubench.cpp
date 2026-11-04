//
// Created by Julia on 28.01.2025.
//

#include <jni.h>

#include <cstdint>
#include <string>
#include <sys/auxv.h>

#ifdef __aarch64__
#include <asm/hwcap.h>
#endif //__aarch64__

#include "ffi.h"
#include "acubench.h"

void throw_runtime_exception(JNIEnv *env, const char *message) {
    jclass clazz = env->FindClass("java/lang/RuntimeException");
    env->ThrowNew(clazz, message);
}

#define THROW_IF_ERR(ENV, REPORT) if (REPORT.err != nullptr) { \
    throw_runtime_exception(ENV, REPORT.err); \
}

Auxval auxval() {
    #ifdef __aarch64__
        uint64_t hwcap = getauxval(AT_HWCAP);
        uint64_t hwcap2 = getauxval(AT_HWCAP2);
        auto sve_mask = TypedU64 {.t = 0, .v = HWCAP_SVE};
        auto i8mm_mask = TypedU64 {.t = 1, .v = HWCAP2_I8MM};
    #else
        uint64_t hwcap = 0;
        uint64_t hwcap2 = 0;
        auto sve_mask = TypedU64 {.t = 0, .v = 0};
        auto i8mm_mask = TypedU64 {.t = 0, .v = 0};
    #endif //__aarch64__

    return Auxval {
        .hwcap = hwcap,
        .hwcap2 = hwcap2,
        .sve_mask = sve_mask,
        .i8mm_mask = i8mm_mask,
    };
}

jobject jcpu_report(JNIEnv *env, const CpuReport &report) {
    jclass clazz = env->FindClass("com/acurast/bench/Acubench$CpuReport");
    jmethodID init = env->GetMethodID(clazz, "<init>", "(DDDD)V");

    return env->NewObject(clazz, init, report.crypto_tps, report.math_tps, report.sort_tps, report.score);
}

extern "C"
JNIEXPORT jobject JNICALL
Java_com_acurast_bench_Acubench__1_1cpu_1_1(JNIEnv *env, jobject thiz,
                                            jlong crypto_duration, jlong crypto_data_len,
                                            jlong math_duration, jlong math_data_len, jboolean math_simd,
                                            jlong sort_duration, jlong sort_data_len) {
    auto report = bench_cpu(auxval(), CpuConfig{
        .crypto_duration = (size_t) crypto_duration,
        .crypto_data_len = (size_t) crypto_data_len,
        .math_duration = (size_t) math_duration,
        .math_data_len = (size_t) math_data_len,
        .math_simd = math_simd == JNI_TRUE,
        .sort_duration = (size_t) sort_duration,
        .sort_data_len = (size_t) sort_data_len
    });
    auto jreport = jcpu_report(env, report);
    THROW_IF_ERR(env, report);

    drop_report_err(report.err);

    return jreport;
}

extern "C"
JNIEXPORT jobject JNICALL
Java_com_acurast_bench_Acubench__1_1cpu_1multithread_1_1(JNIEnv *env, jobject thiz,
                                                         jlong crypto_duration, jlong crypto_data_len,
                                                         jlong math_duration, jlong math_data_len,
                                                         jlong sort_duration, jlong sort_data_len) {
    auto report = bench_cpu_multithread(auxval(), CpuConfig{
        .crypto_duration = (size_t) crypto_duration,
        .crypto_data_len = (size_t) crypto_data_len,
        .math_duration = (size_t) math_duration,
        .math_data_len = (size_t) math_data_len,
        .sort_duration = (size_t) sort_duration,
        .sort_data_len = (size_t) sort_data_len
    });
    auto jreport = jcpu_report(env, report);
    THROW_IF_ERR(env, report);

    drop_report_err(report.err);

    return jreport;
}

jobject jram_report(JNIEnv *env, const RamReport &report) {
    jclass clazz = env->FindClass("com/acurast/bench/Acubench$RamReport");
    jmethodID init = env->GetMethodID(clazz, "<init>", "(JDDDDD)V");

    return env->NewObject(clazz, init, (jlong) report.total_mem, report.alloc_avg_t, report.access_seq_avg_t, report.access_rand_avg_t, report.access_con_avg_t, report.score);
}

extern "C"
JNIEXPORT jobject JNICALL
Java_com_acurast_bench_Acubench__1_1ram_1_1(JNIEnv *env, jobject thiz, jlong total_mem,
                                            jlong alloc_iters, jlong alloc_data_len,
                                            jlong access_seq_iters, jlong access_seq_data_len,
                                            jlong access_rand_iters, jlong access_rand_data_len,
                                            jlong access_concurr_iters, jlong access_concurr_data_len) {
    auto report = bench_ram(auxval(), (uint64_t) total_mem, RamConfig{
        .alloc_iters = (size_t) alloc_iters,
        .alloc_data_len = (size_t) alloc_data_len,
        .access_seq_iters = (size_t) access_seq_iters,
        .access_seq_data_len = (size_t) access_seq_data_len,
        .access_rand_iters = (size_t) access_rand_iters,
        .access_rand_data_len = (size_t) access_rand_data_len,
        .access_concurr_iters = (size_t) access_concurr_iters,
        .access_concurr_data_len = (size_t) access_concurr_data_len,
    });
    auto jreport = jram_report(env, report);
    THROW_IF_ERR(env, report);

    drop_report_err(report.err);

    return jreport;
}

jobject jstorage_report(JNIEnv *env, const StorageReport &report) {
    jclass clazz = env->FindClass("com/acurast/bench/Acubench$StorageReport");
    jmethodID init = env->GetMethodID(clazz, "<init>", "(JDDD)V");

    return env->NewObject(clazz, init, (jlong) report.avail_storage, report.access_seq_avg_t, report.access_rand_avg_t, report.score);
}

extern "C"
JNIEXPORT jobject JNICALL
Java_com_acurast_bench_Acubench__1_1storage_1_1(JNIEnv *env, jobject thiz, jlong avail_storage,
                                                jstring dir, jlong access_seq_iters,
                                                jlong access_seq_data_len_mb,
                                                jlong access_rand_iters,
                                                jlong access_rand_data_len_mb) {
    const char* cdir_const = env->GetStringUTFChars(dir, nullptr);
    std::string cdir(cdir_const);
    env->ReleaseStringUTFChars(dir, cdir_const);

    auto report = bench_storage(auxval(), (uint64_t) avail_storage, StorageConfig{
        .dir = cdir.data(),
        .access_seq_iters = (size_t) access_seq_iters,
        .access_seq_data_len_mb = (size_t) access_seq_data_len_mb,
        .access_rand_iters = (size_t) access_rand_iters,
        .access_rand_data_len_mb = (size_t) access_rand_data_len_mb,
    });
    auto jreport = jstorage_report(env, report);
    THROW_IF_ERR(env, report);

    drop_report_err(report.err);

    return jreport;
}
