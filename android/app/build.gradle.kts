import org.gradle.kotlin.dsl.support.listFilesOrdered

plugins {
    alias(libs.plugins.android.library)
    alias(libs.plugins.kotlin.android)
    alias(libs.plugins.maven.publish)
    alias(libs.plugins.rust.gradle)
}

object Library {
    const val groupId = "com.acurast.bench"
    const val artifactId = "acubench"
    const val version = "1.1.0-beta01"
}

android {
    namespace = "com.acurast.bench"
    compileSdk = 35
    ndkVersion = sdkDirectory.resolve("ndk").listFilesOrdered().last().name

    defaultConfig {
        minSdk = 24
        version = Library.version

        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"

        ndk {
            abiFilters.addAll(setOf("armeabi-v7a", "arm64-v8a"))
        }
        externalNativeBuild {
            cmake {
                targets("acubench")
            }
        }
    }
    buildTypes {
        release {
            isMinifyEnabled = false
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro"
            )
        }
        debug {
            externalNativeBuild {
                cmake {
                    targets("acubenchtest")
                }
            }
        }
    }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_11
        targetCompatibility = JavaVersion.VERSION_11
    }
    kotlinOptions {
        jvmTarget = "11"
    }
    externalNativeBuild {
        cmake {
            path = file("src/main/cpp/CMakeLists.txt")
            version = "3.22.1"
        }
    }
}

kotlin {
    explicitApiWarning()
}

cargo {
    module = "../../rust"
    libname = "acubench"
    targets = listOf("arm", "arm64")
    targetIncludes = arrayOf("")
    profile = "release"
    prebuiltToolchains = true
}

publishing {
    publications {
        register<MavenPublication>("maven") {
            groupId = Library.groupId
            artifactId = Library.artifactId
            version = Library.version

            afterEvaluate {
                from(components["release"])
            }
        }
    }
}

dependencies {
    implementation(libs.androidx.core.ktx)
    testImplementation(libs.junit)
    androidTestImplementation(libs.androidx.junit)
    androidTestImplementation(libs.androidx.espresso.core)
}

val ffiBuild: TaskProvider<Task> = tasks.register("ffiBuild", Task::class.java) {
    dependsOn("cargoBuild")

    doLast {
        val targets = listOf(
            "aarch64-linux-android" to "arm64-v8a",
            "armv7-linux-androideabi" to "armeabi-v7a",
        )

        targets.forEach { (source, destination) ->
            copy {
                from("../../rust/target/$source/release/libacubench.a")
                into("./src/main/cpp/libs/$destination/")
                rename { "libacubench_ffi.a" }
            }
        }
    }
}
