# For more information about using CMake with Android Studio, read the
# documentation: https://d.android.com/studio/projects/add-native-code.html.
# For more examples on how to use CMake, see https://github.com/android/ndk-samples.

# Sets the minimum CMake version required for this project.
cmake_minimum_required(VERSION 3.22.1)

# Declares the project name. The project name can be accessed via ${ PROJECT_NAME},
# Since this is the top level CMakeLists.txt, the project name is also accessible
# with ${CMAKE_PROJECT_NAME} (both CMake variables are in-sync within the top level
# build script scope).
project(acubench)

set(CMAKE_CXX_STANDARD 17)

# Creates and names a library, sets it as either STATIC
# or SHARED, and provides the relative paths to its source code.
# You can define multiple libraries, and CMake builds them for you.
# Gradle automatically packages shared libraries with your APK.
#
# In this top level CMakeLists.txt, ${CMAKE_PROJECT_NAME} is used to define
# the target library name; in the sub-module's CMakeLists.txt, ${PROJECT_NAME}
# is preferred for the same purpose.
#
# In order to load a library into your app from Java/Kotlin, you must call
# System.loadLibrary() and pass the name of the library defined here;
# for GameActivity/NativeActivity derived applications, the same library name must be
# used in the AndroidManifest.xml file.

set(ACUBENCH acubench)
add_library(${ACUBENCH} SHARED
        # List C/C++ source files with relative paths to this CMakeLists.txt.
        src/acubench.h
        src/acubench.cpp

        src/ffi.h
        src/ffi.cpp)

add_library(acubench-ffi SHARED IMPORTED)
set_target_properties(acubench-ffi PROPERTIES IMPORTED_LOCATION ${CMAKE_SOURCE_DIR}/libs/${ANDROID_ABI}/libacubench_ffi.a)

# Specifies libraries CMake should link to your target library. You
# can link libraries from various origins, such as libraries defined in this
# build script, prebuilt third-party libraries, or Android system libraries.
target_link_libraries(${ACUBENCH}
        # List libraries link to the target library
        acubench-ffi
        android
        log)

set(ACUBENCHTEST acubenchtest)
add_library(${ACUBENCHTEST} SHARED
        # List C/C++ source files with relative paths to this CMakeLists.txt.
        test/ffi_test.h
        test/ffi_test.cpp)

target_link_libraries(${ACUBENCHTEST}
        # List libraries link to the target library
        ${ACUBENCH}
        android
        log)