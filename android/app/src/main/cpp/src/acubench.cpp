//
// Created by Julia on 28.01.2025.
//

#include <jni.h>

#include <cstdint>
#include <sys/auxv.h>

#ifdef __aarch64__
#include <asm/hwcap.h>
#endif //__aarch64__

#include <vector>

#include "ffi.h"
#include "acubench.h"

void throw_runtime_exception(JNIEnv *env, const char *message) {
    jclass clazz = env->FindClass("java/lang/RuntimeException");
    env->ThrowNew(clazz, message);
}

#define THROW_IF_ERR(ENV, REPORT, TYPE) if (REPORT->TYPE##_err != nullptr && REPORT->TYPE##_err_len != 0) { \
    throw_runtime_exception(ENV, REPORT->TYPE##_err); \
}

extern "C"
JNIEXPORT jlong JNICALL
Java_com_acurast_bench_Acubench__1_1new_1_1(JNIEnv *env, jobject thiz, jlong total_ram, jlong avail_storage) {
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

    auto bench = new_bench(total_ram, avail_storage, hwcap, hwcap2, sve_mask, i8mm_mask);

    return reinterpret_cast<jlong>(bench);
}

extern "C"
JNIEXPORT void JNICALL
Java_com_acurast_bench_Acubench__1_1delete_1_1(JNIEnv *env, jobject thiz, jlong ptr) {
    drop_bench((void *) ptr);
}

jobject jcpu_report(JNIEnv *env, CpuReport *report) {
    jclass clazz = env->FindClass("com/acurast/bench/Acubench$CpuReport");
    jmethodID init = env->GetMethodID(clazz, "<init>", "(DDD)V");

    return env->NewObject(clazz, init, report->crypto_tps, report->math_tps, report->sort_tps);
}

extern "C"
JNIEXPORT jobject JNICALL
Java_com_acurast_bench_Acubench__1_1cpu_1_1(JNIEnv *env, jobject thiz, jlong ptr,
                                            jlong crypto_duration, jlong crypto_data_len,
                                            jlong math_duration, jlong math_data_len,
                                            jlong sort_duration, jlong sort_data_len) {
    auto report = bench_cpu((void *) ptr, CpuConfig{
        .crypto_duration = (size_t) crypto_duration,
        .crypto_data_len = (size_t) crypto_data_len,
        .math_duration = (size_t) math_duration,
        .math_data_len = (size_t) math_data_len,
        .sort_duration = (size_t) sort_duration,
        .sort_data_len = (size_t) sort_data_len
    });
    auto jreport = jcpu_report(env, report);

    THROW_IF_ERR(env, report, crypto);
    THROW_IF_ERR(env, report, math);
    THROW_IF_ERR(env, report, sort);

    drop_cpu_report(report);

    return jreport;
}

extern "C"
JNIEXPORT jobject JNICALL
Java_com_acurast_bench_Acubench__1_1cpu_1multithread_1_1(JNIEnv *env, jobject thiz, jlong ptr,
                                                         jlong crypto_duration, jlong crypto_data_len,
                                                         jlong math_duration, jlong math_data_len,
                                                         jlong sort_duration, jlong sort_data_len) {
    auto report = bench_cpu_multithread((void *) ptr, CpuConfig{
        .crypto_duration = (size_t) crypto_duration,
        .crypto_data_len = (size_t) crypto_data_len,
        .math_duration = (size_t) math_duration,
        .math_data_len = (size_t) math_data_len,
        .sort_duration = (size_t) sort_duration,
        .sort_data_len = (size_t) sort_data_len
    });

    auto jreport = jcpu_report(env, report);

    THROW_IF_ERR(env, report, crypto);
    THROW_IF_ERR(env, report, math);
    THROW_IF_ERR(env, report, sort);

    drop_cpu_report(report);

    return jreport;
}

jobject jram_report(JNIEnv *env, RamReport *report) {
    jclass clazz = env->FindClass("com/acurast/bench/Acubench$RamReport");
    jmethodID init = env->GetMethodID(clazz, "<init>", "(JDDDD)V");

    return env->NewObject(clazz, init, (jlong) report->total_mem, report->alloc_avg_t, report->access_seq_avg_t, report->access_rand_avg_t, report->access_con_avg_t);
}

extern "C"
JNIEXPORT jobject JNICALL
Java_com_acurast_bench_Acubench__1_1ram_1_1(JNIEnv *env, jobject thiz, jlong ptr, jlong alloc_iters,
                                            jlong alloc_data_len, jlong access_seq_iters,
                                            jlong access_seq_data_len, jlong access_rand_iters,
                                            jlong access_rand_data_len, jlong access_concurr_iters,
                                            jlong access_concurr_data_len) {
    auto report = bench_ram((void *) ptr, RamConfig{
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

    THROW_IF_ERR(env, report, alloc);
    THROW_IF_ERR(env, report, access);

    drop_ram_report(report);

    return jreport;
}

jobject jstorage_report(JNIEnv *env, StorageReport *report) {
    jclass clazz = env->FindClass("com/acurast/bench/Acubench$StorageReport");
    jmethodID init = env->GetMethodID(clazz, "<init>", "(JDD)V");

    return env->NewObject(clazz, init, (jlong) report->avail_storage, report->access_seq_avg_t, report->access_rand_avg_t);
}

extern "C"
JNIEXPORT jobject JNICALL
Java_com_acurast_bench_Acubench__1_1storage_1_1(JNIEnv *env, jobject thiz, jlong ptr,
                                                jbyteArray dir, jlong access_seq_iters,
                                                jlong access_seq_data_len_mb,
                                                jlong access_rand_iters,
                                                jlong access_rand_data_len_mb) {
    jsize dir_len = env->GetArrayLength(dir);
    jbyte *jdir = env->GetByteArrayElements(dir, nullptr);
    std::vector<char> dir_vec(reinterpret_cast<char*>(jdir), reinterpret_cast<char*>(jdir) + dir_len);
    env->ReleaseByteArrayElements(dir, jdir, JNI_ABORT);

    auto report = bench_storage((void *) ptr, StorageConfig{
        .dir = dir_vec.data(),
        .dir_len = dir_vec.size(),
        .access_seq_iters = (size_t) access_seq_iters,
        .access_seq_data_len_mb = (size_t) access_seq_data_len_mb,
        .access_rand_iters = (size_t) access_rand_iters,
        .access_rand_data_len_mb = (size_t) access_rand_data_len_mb,
    });

    auto jreport = jstorage_report(env, report);

    THROW_IF_ERR(env, report, access);

    drop_storage_report(report);

    return jreport;
}