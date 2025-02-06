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
Java_com_acurast_bench_Acubench__1_1cpu_1_1(JNIEnv *env, jobject thiz, jlong ptr, jlong duration,
                                        jlong enc_data_len, jlong math_data_len, jlong sort_data_len) {

    auto report = bench_cpu((void *) ptr, CpuConfig{
        .duration = (size_t) duration,
        .enc_data_len = (size_t) enc_data_len,
        .math_data_len = (size_t) math_data_len,
        .sort_data_len = (size_t) sort_data_len
    });
    auto jreport = jcpu_report(env, report);
    if (report->err != nullptr && report->err_len != 0) {
        throw_runtime_exception(env, report->err);
    }
    drop_cpu_report(report);

    return jreport;
}

extern "C"
JNIEXPORT jobject JNICALL
Java_com_acurast_bench_Acubench__1_1cpu_1multithread_1_1(JNIEnv *env, jobject thiz, jlong ptr,
                                                         jlong duration, jlong enc_data_len,
                                                         jlong math_data_len, jlong sort_data_len) {
    auto report = bench_cpu_multithread((void *) ptr, CpuConfig{
        .duration = (size_t) duration,
        .enc_data_len = (size_t) enc_data_len,
        .math_data_len = (size_t) math_data_len,
        .sort_data_len = (size_t) sort_data_len
    });

    auto jreport = jcpu_report(env, report);
    if (report->err != nullptr && report->err_len != 0) {
        throw_runtime_exception(env, report->err);
    }
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
Java_com_acurast_bench_Acubench__1_1ram_1_1(JNIEnv *env, jobject thiz, jlong ptr, jlong alloc_data_len,
                                            jlong access_data_len, jlong iters) {
    auto report = bench_ram((void *) ptr, RamConfig{
        .alloc_data_len = (size_t) alloc_data_len,
        .access_data_len = (size_t) access_data_len,
        .iters = (size_t) iters
    });

    auto jreport = jram_report(env, report);
    if (report->err != nullptr && report->err_len != 0) {
        throw_runtime_exception(env, report->err);
    }
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
Java_com_acurast_bench_Acubench__1_1storage_1_1(JNIEnv *env, jobject thiz, jlong ptr, jbyteArray dir,
                                                jlong access_data_len_mb, jlong iters) {
    jsize dir_len = env->GetArrayLength(dir);
    jbyte *jdir = env->GetByteArrayElements(dir, nullptr);
    std::vector<char> dir_vec(reinterpret_cast<char*>(jdir), reinterpret_cast<char*>(jdir) + dir_len);
    env->ReleaseByteArrayElements(dir, jdir, JNI_ABORT);

    auto report = bench_storage((void *) ptr, StorageConfig{
        .dir = dir_vec.data(),
        .dir_len = dir_vec.size(),
        .access_data_len_mb = (size_t) access_data_len_mb,
        .iters = (size_t) iters
    });

    auto jreport = jstorage_report(env, report);
    if (report->err != nullptr && report->err_len != 0) {
        throw_runtime_exception(env, report->err);
    }
    drop_storage_report(report);

    return jreport;
}