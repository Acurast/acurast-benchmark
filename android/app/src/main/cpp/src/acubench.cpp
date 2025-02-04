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
Java_com_acurast_bench_Acubench__1_1new_1_1(JNIEnv *env, jobject thiz) {
    auto hwcap = getauxval(AT_HWCAP);
    auto hwcap2 = getauxval(AT_HWCAP2);

    return (jlong) new_bench(hwcap, hwcap2, TypedU64{.t = 0, .v = HWCAP_SVE},
                             TypedU64{.t = 1, .v = HWCAP2_I8MM});
}

extern "C"
JNIEXPORT void JNICALL
Java_com_acurast_bench_Acubench__1_1delete_1_1(JNIEnv *env, jobject thiz, jlong ptr) {
    drop_bench((void *) ptr);
}

jdoubleArray map_cpu_report(JNIEnv *env, CpuReport *report) {
    if (report->err != nullptr && report->err_len != 0) {
        throw_runtime_exception(env, report->err);
        return nullptr;
    }

    jdoubleArray tps = env->NewDoubleArray(3);
    if (tps == nullptr) {
        return nullptr;
    }

    jdouble buf[] = {report->crypto_tps, report->math_tps, report->sort_tps};
    env->SetDoubleArrayRegion(tps, 0, 3, buf);

    return tps;
}

extern "C"
JNIEXPORT jdoubleArray JNICALL
Java_com_acurast_bench_Acubench__1_1cpu_1_1(JNIEnv *env, jobject thiz, jlong ptr, jlong duration,
                                        jlong enc_data_len, jlong math_data_len, jlong sort_data_len) {

    auto report = bench_cpu((void *) ptr, CpuConfig{
            .duration = (size_t) duration,
            .enc_data_len = (size_t) enc_data_len,
            .math_data_len = (size_t) math_data_len,
            .sort_data_len = (size_t) sort_data_len
    });

    auto arr = map_cpu_report(env, report);
    drop_cpu_report(report);

    return arr;
}

extern "C"
JNIEXPORT jdoubleArray JNICALL
Java_com_acurast_bench_Acubench__1_1cpu_1multithread_1_1(JNIEnv *env, jobject thiz, jlong ptr,
                                                         jlong duration, jlong enc_data_len,
                                                         jlong math_data_len, jlong sort_data_len) {
    auto report = bench_cpu_multithread((void *) ptr, CpuConfig{
            .duration = (size_t) duration,
            .enc_data_len = (size_t) enc_data_len,
            .math_data_len = (size_t) math_data_len,
            .sort_data_len = (size_t) sort_data_len
    });

    auto arr = map_cpu_report(env, report);
    drop_cpu_report(report);

    return arr;
}