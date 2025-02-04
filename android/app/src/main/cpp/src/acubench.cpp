//
// Created by Julia on 28.01.2025.
//

#include <jni.h>

#include <cstdint>
#include <sys/auxv.h>
#include <asm/hwcap.h>

#include "ffi.h"
#include "acubench.h"

void throw_runtime_exception(JNIEnv *env, const char *message) {
    jclass clazz = env->FindClass("java/lang/RuntimeException");
    env->ThrowNew(clazz, message);
}

extern "C"
JNIEXPORT jlong JNICALL
Java_com_acurast_bench_Acubench__1_1new_1_1(JNIEnv *env, jobject thiz, jlong total_ram) {
    auto hwcap = getauxval(AT_HWCAP);
    auto hwcap2 = getauxval(AT_HWCAP2);

    auto bench = new_bench(total_ram, hwcap, hwcap2, TypedU64{.t = 0, .v = HWCAP_SVE},
                             TypedU64{.t = 1, .v = HWCAP2_I8MM});

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
