# Acurast Benchmark

[![jitpack](https://img.shields.io/jitpack/v/github/acurast/acurast-benchmark)](https://jitpack.io/#acurast/acurast-benchmark)
[![spm](https://img.shields.io/github/v/tag/acurast/acurast-benchmark?include_prereleases&label=spm)](https://github.com/acurast/acurast-benchmark/releases)

The Acurast Benchmark suite is designed to evaluate the performance of processors running in the Acurast network. It provides a comprehensive set of tests to measure computational throughput, memory allocation and access efficiency, and storage read/write performance. The benchmarks include support for multithreaded execution to leverage modern multi-core architectures.

## Benchmarks

### CPU
The CPU benchmarks evaluate the computational capabilities of the device. They include:
- **Crypto**: Measures encryption and decryption throughput using [AES-256](https://en.wikipedia.org/wiki/Advanced_Encryption_Standard) and hashing throughput using [SHA-256](https://en.wikipedia.org/wiki/Secure_Hash_Algorithms).
- **Math**: Tests matrix multiplication performance, with support for three algorithms:
    - [**Divide and Conquer**](https://en.wikipedia.org/wiki/Matrix_multiplication_algorithm#Divide-and-conquer_algorithm): A recursive algorithm used for single-threaded execution when SIMD is disabled or for multithreaded benchmarks in general.
    - [**Iterative**](https://en.wikipedia.org/wiki/Matrix_multiplication_algorithm#Iterative_algorithm): A straightforward implementation used when SIMD optimization is enabled but the device lacks the required hardware capabilities.
    - **SIMD-Optimized**: An optimized version of the iterative algorithm leveraging SIMD instructions, used when the device supports the required hardware capabilities.
- **Sort**: Benchmarks sorting algorithms, including single-threaded and multithreaded [merge sort](https://en.wikipedia.org/wiki/Merge_sort).

The CPU benchmark suite is available in two variations:
- **Single-Core**: Executes all tests using a single thread to evaluate the performance of a single core.
- **Multi-Core**: Executes all tests using multiple threads to leverage the full computational power of the device.

### RAM
The RAM benchmarks assess memory allocation and access patterns:
- **Allocation**: Measures the time taken to allocate and initialize memory.
- **Access**: Evaluates sequential, random, and concurrent memory access patterns.

### Storage
The storage benchmarks focus on file I/O performance:
- **Access**: Measures sequential and random read/write throughput for files on the storage medium.

## Methodology

### CPU

#### Configuration
- **Crypto**: Configure the duration of the test and the size of the data to encrypt and hash.
- **Math**: Specify the matrix size, whether to enable SIMD optimizations, and the duration of the test.
- **Sort**: Define the size of the dataset and the duration of the test.

#### Execution
- **Crypto**: The test encrypts and decrypts data using AES-256 and computes hashes using SHA-256.
- **Math**: Performs matrix multiplication using one of three algorithms:
    - **Divide and Conquer**: Used for single-threaded execution when SIMD is disabled or for multithreaded benchmarks.
    - **Iterative**: Used when SIMD is enabled but the device lacks the required hardware capabilities.
    - **SIMD-Optimized**: Used when SIMD is enabled and the device supports the required hardware capabilities.
- **Sort**: Executes merge sort on a dataset, either single-threaded or multithreaded.

#### Results
- **Crypto**: Reports the throughput in bytes per second for encryption and hashing.
- **Math**: Outputs the number of operations per second for matrix multiplication.
- **Sort**: Provides the number of sorted elements per second.

#### Evaluation
- **Score**: The CPU score is calculated as the average of the throughput ($`TPS`$) values from the Crypto, Math, and Sort benchmarks. Higher throughput results in a higher score.

```math
score_{CPU} = \frac{TPS_{crypto} + TPS_{math} + TPS_{sort}}{3}
```

### RAM

#### Configuration
- **Allocation**: Specify the number of iterations and the size of memory to allocate in each iteration.
- **Access**: Configure the number of iterations and the size of data for sequential, random, and concurrent access patterns.

#### Execution
- **Allocation**: Allocates and initializes memory repeatedly to measure allocation performance.
- **Access**: Reads and writes data in sequential, random, and concurrent patterns to evaluate memory access efficiency.

#### Results
- **Allocation**: Reports the average time taken to allocate and initialize memory.
- **Access**: Provides average times for sequential, random, and concurrent access patterns.

#### Evaluation
- **Total RAM**: The total available RAM memory.
- **Score**: The RAM score is calculated based on the inverse of the average times ($`T`$) for Allocation and Access benchmarks. Higher efficiency results in a higher score.

```math
score_{RAM} = \frac{T_{alloc}^{-1} + T_{access\_seq}^{-1} + T_{access\_rand}^{-1} + T_{access\_concurr}^{-1}}{4}
```

### Storage

#### Configuration
- **Access**: Define the number of iterations and the size of data for sequential and random read/write operations.

#### Execution
- **Access**: Creates temporary files and performs sequential and random read/write operations. Files are removed after each iteration.

#### Results
- **Access**: Reports the average time for sequential and random read/write operations.

#### Evaluation
- **Available Storage**: The available storage capacity.
- **Score**: The Storage score is calculated as the average of the inversed average times ($`T`$) for sequential and random read/write operations. Lower average times result in a higher score.

```math
score_{storage} = \frac{T_{access\_seq}^{-1} + T_{access\_rand}^{-1}}{2}
```

## Usage

### Android

#### Adding the Dependency

To use the Acurast Benchmark library in your Android project, add the following to your `settings.gradle` file to enable Jitpack:

```kotlin
dependencyResolutionManagement {
    repositories {
        maven { url = uri("https://jitpack.io") }
        google()
        mavenCentral()
    }
}
```

Then, add the dependency to your `build.gradle` file:

```kotlin
dependencies {
    implementation("com.github.acurast:acurast-benchmark:<version>")
}
```

Replace `<version>` with the latest version available on [Jitpack](https://jitpack.io/#acurast/acurast-benchmark).

#### Using the Library

1. **Initialize the Library**  
    Create an instance of `Acubench`:
    ```kotlin
    import com.acurast.bench.Acubench

    val acubench = Acubench(context)
    ```

2. **Run Benchmarks**  
   Run the desired benchmarks using the provided methods:
    ```kotlin 
    fun cpuSingleCore() {
        val report = acubench.cpu()

        println("""
             CPU Single-Core
             :::: crypto ${report.cryptoTps} ops/s
             :::: math   ${report.mathTps} ops/s
             :::: sort   ${report.sortTps} ops/s
             ----
             :::: score  ${report.score}
        """.trimIndent())
    }

    fun cpuMultiCore() {
        val report = acubench.cpuMultithread()

        println("""
             CPU Multi-Core
             :::: crypto ${report.cryptoTps} ops/s
             :::: math   ${report.mathTps} ops/s
             :::: sort   ${report.sortTps} ops/s
             ----
             :::: score  ${report.score}
        """.trimIndent())
    }

    fun ram() {
        val report = acubench.ram()

        println("""
            RAM
            :::: total memory        ${report.totalMemory / 1024f / 1024f} GB
            :::: alloc               ${report.allocAvgTime} s
            :::: access (sequential) ${report.accessSequentialAvgTime} s
            :::: access (random)     ${report.accessRandomAvgTime} s
            :::: access (concurrent) ${report.accessConcurrentAvgTime} s
            ----
            :::: score               ${report.score}
        """.trimIndent())
    }

    fun storage() {
        val storage = acubench.storage(context)

        println("""
            Storage
            :::: available           ${report.availableStorage / 1024f / 1024f} GB
            :::: access (sequential) ${report.accessSequentialAvgTime} s
            :::: access (random)     ${report.accessRandomAvgTime} s
            ----
            :::: score               ${report.score}
        """.trimIndent())
    }

    cpuSingleCore()
    cpuMultiCore()
    ram()
    storage()
    ```

3. **Clean Up**  
    Release the resources when done:
    ```kotlin
    acubench.destroy()
    ```

#### Configuration

The configuration structures allow you to customize the behavior of the benchmarks. Each structure provides fields to fine-tune the parameters for specific tests. If not specified, default values will be used.

- **CpuConfig**: Configures the CPU benchmark.
  ```kotlin
  data class CpuConfig(
      val cryptoDuration: Duration = 1.seconds, // Duration of the crypto test
      val cryptoDataSize: Long = 10 * 1024,     // Size of data to encrypt and hash (in bytes)
      val mathDuration: Duration = 1.seconds,   // Duration of the math test
      val mathDataSize: Long = 200,             // Size of the matrix for multiplication
      val mathSimd: Boolean = true,             // Enable SIMD optimizations
      val sortDuration: Duration = 1.seconds,   // Duration of the sort test
      val sortDataSize: Long = 100_000          // Number of elements to sort
  )
  ```

- **RamConfig**: Configures the RAM benchmark.
  ```kotlin
  data class RamConfig(
      val allocIters: Long = 10,                      // Number of iterations for memory allocation
      val allocDataSize: Long = 64 * 1024 * 1024,     // Size of memory to allocate per iteration (in bytes)
      val accessSequentialIters: Long = 10,           // Number of iterations for sequential access
      val accessSequentialDataSize: Long = 64 * 1024, // Size of data for sequential access (in bytes)
      val accessRandomIters: Long = 10,               // Number of iterations for random access
      val accessRandomDataSize: Long = 64 * 1024,     // Size of data for random access (in bytes)
      val accessConcurrentIters: Long = 10,           // Number of iterations for concurrent access
      val accessConcurrentDataSize: Long = 64 * 1024  // Size of data for concurrent access (in bytes)
  )
  ```

- **StorageConfig**: Configures the Storage benchmark.
  ```kotlin
  data class StorageConfig(
      val dir: File,                              // Directory to perform storage tests
      val accessSequentialIters: Long = 1,        // Number of iterations for sequential access
      val accessSequentialDataSizeMB: Long = 50,  // Size of data for sequential access (in MB)
      val accessRandomIters: Long = 1,            // Number of iterations for random access
      val accessRandomDataSizeMB: Long = 50       // Size of data for random access (in MB)
  )
  ```

#### Results

The report structures provide detailed results of the benchmarks.

- **CpuReport**: Contains CPU benchmark results.
  ```kotlin
  data class CpuReport(
      val cryptoTps: Double, // Throughput for crypto operations (ops/s)
      val mathTps: Double,   // Throughput for matrix multiplication (ops/s)
      val sortTps: Double,   // Throughput for sorting operations (ops/s)
      val score: Double      // Overall CPU performance score
  )
  ```

- **RamReport**: Contains RAM benchmark results.
  ```kotlin
  data class RamReport(
      val totalMemory: Long,               // Total available RAM (in bytes)
      val allocAvgTime: Double,            // Average time for memory allocation (in seconds)
      val accessSequentialAvgTime: Double, // Average time for sequential memory access (in seconds)
      val accessRandomAvgTime: Double,     // Average time for random memory access (in seconds)
      val accessConcurrentAvgTime: Double, // Average time for concurrent memory access (in seconds)
      val score: Double                    // Overall RAM performance score
  )
  ```

- **StorageReport**: Contains Storage benchmark results.
  ```kotlin
  data class StorageReport(
      val availableStorage: Long,          // Available storage space (in bytes)
      val accessSequentialAvgTime: Double, // Average time for sequential storage access (in seconds)
      val accessRandomAvgTime: Double,     // Average time for random storage access (in seconds)
      val score: Double                    // Overall Storage performance score
  )
  ```

### iOS

#### Adding the Dependency

To use the Acurast Benchmark library in your iOS project, add the following to your `Package.swift` file to fetch the library via Swift Package Manager (SPM):

```swift
dependencies: [
    .package(url: "https://github.com/acurast/acurast-benchmark.git", .upToNextMajor(from: "<version>"))
]
```

Replace `<version>` with the latest version available on [GitHub Releases](https://github.com/acurast/acurast-benchmark/releases).

Then, add the library as a dependency to your target:

```swift
.target(
    name: "TargetName",
    dependencies: [
        .product(name: "Acubench", package: "acurast-benchmark")
    ]
)
```

#### Using the Library

1. **Initialize the Library**  
   Create an instance of `Acubench`:
   ```swift
   import Acubench

   let acubench = Acubench()
   ```

2. **Run Benchmarks**  
   Run the desired benchmarks using the provided methods:
   ```swift
   func cpuSingleCore() throws {
        let report = try acubench.cpu()

        print(
            """
            CPU Single-Core
            :::: crypto \(report.cryptoTPS.formatted()) ops/s
            :::: math   \(report.mathTPS.formatted()) ops/s
            :::: sort   \(report.sortTPS.formatted()) ops/s
            ----
            :::: score  \(report.score.formatted())
            """)
    }

    func cpuMultiCore() throws {
        let report = try acubench.cpuMultithread()

        print(
            """
            CPU Multi-Core
            :::: crypto \(report.cryptoTPS.formatted()) ops/s
            :::: math   \(report.mathTPS.formatted()) ops/s
            :::: sort   \(report.sortTPS.formatted()) ops/s
            ----
            :::: score  \(report.score.formatted())
            """)
    }

    func ram() throws {
        let report = try acubench.ram()

        print(
            """
            RAM
            :::: total memory        \(report.totalMemory / 1024 / 1024) GB
            :::: alloc               \(report.allocAvgTime.formatted()) s
            :::: access (sequential) \(report.accessSequentialAvgTime.formatted()) s
            :::: access (random)     \(report.accessRandomAvgTime.formatted()) s
            :::: access (concurrent) \(report.accessConcurrentAvgTime.formatted()) s
            ----
            :::: score               \(report.score.formatted())
            """)
    }

    func storage() throws {
        let report = try acubench.storage()

        print(
            """
            Storage
            :::: available           \(report.availableStorage / 1024 / 1024 / 1024) GB
            :::: access (sequential) \(report.accessSequentialAvgTime.formatted()) s
            :::: access (random)     \(report.accessRandomAvgTime.formatted()) s
            ----
            :::: score               \(report.score.formatted())
            """)
    }

    try cpuSingleCore()
    try cpuMultiCore()
    try ram()
    try storage()
   ```

#### Configuration

The configuration structures allow you to customize the behavior of the benchmarks. Each structure provides fields to adjust the parameters for specific tests.

- **CPUConfig**: Configures the CPU benchmark.
  ```swift
  struct CPUConfig {
      let cryptoDuration: TimeInterval = 1.0 // Duration of the crypto test (in seconds)
      let cryptoDataSize: Int = 10 * 1024    // Size of data to encrypt and hash (in bytes)
      let mathDuration: TimeInterval = 1.0   // Duration of the math test (in seconds)
      let mathDataSize: Int = 200            // Size of the matrix for multiplication
      let mathSimd: Bool = true              // Enable SIMD optimizations
      let sortDuration: TimeInterval = 1.0   // Duration of the sort test (in seconds)
      let sortDataSize: Int = 100_000        // Number of elements to sort
  }
  ```

- **RAMConfig**: Configures the RAM benchmark.
  ```swift
  struct RAMConfig {
      let allocIters: Int = 10                      // Number of iterations for memory allocation
      let allocDataSize: Int = 64 * 1024 * 1024     // Size of memory to allocate per iteration (in bytes)
      let accessSequentialIters: Int = 10           // Number of iterations for sequential access
      let accessSequentialDataSize: Int = 64 * 1024 // Size of data for sequential access (in bytes)
      let accessRandomIters: Int = 10               // Number of iterations for random access
      let accessRandomDataSize: Int = 64 * 1024     // Size of data for random access (in bytes)
      let accessConcurrentIters: Int = 10           // Number of iterations for concurrent access
      let accessConcurrentDataSize: Int = 64 * 1024 // Size of data for concurrent access (in bytes)
  }
  ```

- **StorageConfig**: Configures the Storage benchmark.
  ```swift
  struct StorageConfig {
      let dir: URL                              // Directory to perform storage tests
      let accessSequentialIters: Int = 1        // Number of iterations for sequential access
      let accessSequentialDataSizeMB: Int = 50  // Size of data for sequential access (in MB)
      let accessRandomIters: Int = 1            // Number of iterations for random access
      let accessRandomDataSizeMB: Int = 50      // Size of data for random access (in MB)
  }
  ```

#### Reports

The report structures provide detailed results of the benchmarks.

- **CPUReport**: Contains results for the CPU benchmark.
  ```swift
  struct CPUReport {
      let cryptoTPS: Double // Throughput for crypto operations (ops/s)
      let mathTPS: Double   // Throughput for matrix multiplication (ops/s)
      let sortTPS: Double   // Throughput for sorting operations (ops/s)
      let score: Double     // Overall CPU performance score
  }
  ```

- **RAMReport**: Contains results for the RAM benchmark.
  ```swift
  struct RAMReport {
      let totalMemory: Int64               // Total available RAM (in bytes)
      let allocAvgTime: Double             // Average time for memory allocation (in seconds)
      let accessSequentialAvgTime: Double  // Average time for sequential memory access (in seconds)
      let accessRandomAvgTime: Double      // Average time for random memory access (in seconds)
      let accessConcurrentAvgTime: Double  // Average time for concurrent memory access (in seconds)
      let score: Double                    // Overall RAM performance score
  }
  ```

- **StorageReport**: Contains results for the Storage benchmark.
  ```swift
  struct StorageReport {
      let availableStorage: Int64          // Available storage space (in bytes)
      let accessSequentialAvgTime: Double  // Average time for sequential storage access (in seconds)
      let accessRandomAvgTime: Double      // Average time for random storage access (in seconds)
      let score: Double                    // Overall Storage performance score
  }
  ```