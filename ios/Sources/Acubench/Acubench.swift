//
//  Acubench.swift
//  Acubench
//
//  Created by Pablo Martinez Piles on 6/3/25.
//

import Foundation
import AcubenchFFI

private let KB: Int = 1024
private let MB: Int = KB * 1024

public struct TypedU64 {
    public let t: UInt8
    public let v: UInt64
    
    public init(t: UInt8, v: UInt64) {
        self.t = t
        self.v = v
    }
    
    fileprivate func toCStruct() -> AcubenchFFI.TypedU64 {
        var cStruct = AcubenchFFI.TypedU64()
        cStruct.t = t
        cStruct.v = v
        
        return cStruct
    }
}

public struct CPUConfig: Config {
    fileprivate typealias CStruct = AcubenchFFI.CpuConfig
    
    public let cryptoDuration: Int
    public let cryptoDataSize: Int
    public let mathDuration: Int
    public let mathDataSize: Int
    public let mathSIMD: Bool
    public let sortDuration: Int
    public let sortDataSize: Int
    
    public init(
        cryptoDuration: Int,
        cryptoDataSize: Int = Self.Default.cryptoDataSize,
        mathDuration: Int,
        mathDataSize: Int = Self.Default.mathDataSize,
        mathSIMD: Bool = Self.Default.mathSIMD,
        sortDuration: Int,
        sortDataSize: Int = Self.Default.sortDataSize
    ) {
        self.cryptoDuration = cryptoDuration
        self.cryptoDataSize = cryptoDataSize
        self.mathDuration = mathDuration
        self.mathDataSize = mathDataSize
        self.mathSIMD = mathSIMD
        self.sortDuration = sortDuration
        self.sortDataSize = sortDataSize
    }
    
    public init(
        duration: Int,
        cryptoDataSize: Int = Self.Default.cryptoDataSize,
        mathDataSize: Int = Self.Default.mathDataSize,
        mathSIMD: Bool = Self.Default.mathSIMD,
        sortDataSize: Int = Self.Default.sortDataSize
    ) {
        self.init(
            cryptoDuration: duration,
            cryptoDataSize: cryptoDataSize,
            mathDuration: duration,
            mathDataSize: mathDataSize,
            mathSIMD: mathSIMD,
            sortDuration: duration,
            sortDataSize: sortDataSize
        )
    }
    
    public init(
        cryptoDuration: TimeInterval = TimeInterval(Self.Default.duration / 1000),
        cryptoDataSize: Int = Self.Default.cryptoDataSize,
        mathDuration: TimeInterval = TimeInterval(Self.Default.duration / 1000),
        mathDataSize: Int = Self.Default.mathDataSize,
        mathSIMD: Bool = Self.Default.mathSIMD,
        sortDuration: TimeInterval = TimeInterval(Self.Default.duration / 1000),
        sortDataSize: Int = Self.Default.sortDataSize
    ) {
        self.init(
            cryptoDuration: Int(cryptoDuration * 1000),
            cryptoDataSize: cryptoDataSize,
            mathDuration: Int(mathDuration * 1000),
            mathDataSize: mathDataSize,
            mathSIMD: mathSIMD,
            sortDuration: Int(sortDuration * 1000),
            sortDataSize: sortDataSize
        )
    }
    
    public init(
        duration: TimeInterval = TimeInterval(Self.Default.duration / 1000),
        cryptoDataSize: Int = Self.Default.cryptoDataSize,
        mathDataSize: Int = Self.Default.mathDataSize,
        mathSIMD: Bool = Self.Default.mathSIMD,
        sortDataSize: Int = Self.Default.sortDataSize
    ) {
        self.init(
            cryptoDuration: duration,
            cryptoDataSize: cryptoDataSize,
            mathDuration: duration,
            mathDataSize: mathDataSize,
            mathSIMD: mathSIMD,
            sortDuration: duration,
            sortDataSize: sortDataSize
        )
    }
    
    fileprivate func toCStruct() -> AcubenchFFI.CpuConfig {
        var cStruct = AcubenchFFI.CpuConfig()
        cStruct.crypto_duration = cryptoDuration
        cStruct.crypto_data_len = cryptoDataSize
        cStruct.math_duration = mathDuration
        cStruct.math_data_len = mathDataSize
        cStruct.math_simd = mathSIMD
        cStruct.sort_duration = sortDuration
        cStruct.sort_data_len = sortDataSize
        
        return cStruct
    }
    
    public struct Default {
        public static let duration: Int = 1000
        public static let cryptoDataSize: Int = 10 * KB
        public static let mathDataSize: Int = 200
        public static let mathSIMD: Bool = true
        public static let sortDataSize: Int = 100_000
    }
}

public struct CPUReport: Report {
    fileprivate typealias CStruct = AcubenchFFI.CpuReport
    
    public let cryptoTPS: Double
    public let mathTPS: Double
    public let sortTPS: Double
    public let score: Double
    
    public let error: String?
    
    fileprivate init(from cStruct: UnsafeMutablePointer<AcubenchFFI.CpuReport>) {
        cryptoTPS = cStruct.pointee.crypto_tps
        mathTPS = cStruct.pointee.math_tps
        sortTPS = cStruct.pointee.sort_tps
        score = cStruct.pointee.score
        if let err = cStruct.pointee.err {
            error = String(cString: err)
        } else {
            error = nil
        }
        drop_cpu_report(cStruct)
    }
}


@available(macOS 12.0, *)
extension CPUReport: CustomDebugStringConvertible {
    public var debugDescription: String {
        """
        CPU
        :::: crypto \(cryptoTPS.formattedWithPrecision()) ops/s
        :::: math   \(mathTPS.formattedWithPrecision()) ops/s
        :::: sort   \(sortTPS.formattedWithPrecision()) ops/s
        ----
        :::: score  \(score.formattedWithPrecision())
        """
    }
}


public struct RAMConfig: Config {
    fileprivate typealias CStruct = AcubenchFFI.RamConfig
    
    public let allocIters: Int
    public let allocDataSize: Int
    public let accessSequentialIters: Int
    public let accessSequentialDataSize: Int
    public let accessRandomIters: Int
    public let accessRandomDataSize: Int
    public let accessConcurrentIters: Int
    public let accessConcurrentDataSize: Int
    
    public init(
        allocIters: Int = Self.Default.iters,
        allocDataSize: Int = Self.Default.allocDataSize,
        accessSequentialIters: Int = Self.Default.iters,
        accessSequentialDataSize: Int = Self.Default.accessDataSize,
        accessRandomIters: Int = Self.Default.iters,
        accessRandomDataSize: Int = Self.Default.accessDataSize,
        accessConcurrentIters: Int = Self.Default.iters,
        accessConcurrentDataSize: Int = Self.Default.accessDataSize
    ) {
        self.allocIters = allocIters
        self.allocDataSize = allocDataSize
        self.accessSequentialIters = accessSequentialIters
        self.accessSequentialDataSize = accessSequentialDataSize
        self.accessRandomIters = accessRandomIters
        self.accessRandomDataSize = accessRandomDataSize
        self.accessConcurrentIters = accessConcurrentIters
        self.accessConcurrentDataSize = accessConcurrentDataSize
    }
    
    public init(
        allocIters: Int = Self.Default.iters,
        allocDataSize: Int = Self.Default.allocDataSize,
        accessIters: Int = Self.Default.iters,
        accessDataSize: Int = Self.Default.accessDataSize
    ) {
        self.init(
            allocIters: allocIters,
            allocDataSize: allocDataSize,
            accessSequentialIters: accessIters,
            accessSequentialDataSize: accessDataSize,
            accessRandomIters: accessIters,
            accessRandomDataSize: accessDataSize,
            accessConcurrentIters: accessIters,
            accessConcurrentDataSize: accessDataSize
        )
    }
    
    public init(
        iters: Int = Self.Default.iters,
        allocDataSize: Int = Self.Default.allocDataSize,
        accessDataSize: Int = Self.Default.accessDataSize
    ) {
        self.init(
            allocIters: iters,
            allocDataSize: allocDataSize,
            accessIters: iters,
            accessDataSize: accessDataSize
        )
    }
    
    fileprivate func toCStruct() -> AcubenchFFI.RamConfig {
        var cStruct = AcubenchFFI.RamConfig()
        cStruct.alloc_iters = allocIters
        cStruct.alloc_data_len = allocDataSize
        cStruct.access_seq_iters = accessSequentialIters
        cStruct.access_seq_data_len = accessSequentialDataSize
        cStruct.access_rand_iters = accessRandomIters
        cStruct.access_rand_data_len = accessRandomDataSize
        cStruct.access_concurr_iters = accessConcurrentIters
        cStruct.access_concurr_data_len = accessConcurrentDataSize
        
        return cStruct
    }
    
    public struct Default {
        public static let iters: Int = 10
        public static let allocDataSize: Int = 64 * MB
        public static let accessDataSize: Int = 64 * KB
    }
}

public struct RAMReport: Report {
    fileprivate typealias CStruct = AcubenchFFI.RamReport
    
    public let totalMemory: UInt
    public let allocAvgTime: Double
    public let accessSequentialAvgTime: Double
    public let accessRandomAvgTime: Double
    public let accessConcurrentAvgTime: Double
    public let score: Double
    
    public let error: String?
    
    fileprivate init(from cStruct: UnsafeMutablePointer<AcubenchFFI.RamReport>) {
        totalMemory = UInt(cStruct.pointee.total_mem)
        allocAvgTime = cStruct.pointee.alloc_avg_t
        accessSequentialAvgTime = cStruct.pointee.access_seq_avg_t
        accessRandomAvgTime = cStruct.pointee.access_rand_avg_t
        accessConcurrentAvgTime = cStruct.pointee.access_con_avg_t
        score = cStruct.pointee.score
        if let err = cStruct.pointee.err {
            error = String(cString: err)
        } else {
            error = nil
        }
        drop_ram_report(cStruct)
    }
}

@available(macOS 12.0, *)
extension RAMReport: CustomDebugStringConvertible {
    public var debugDescription: String {
        """
        RAM
        :::: total memory        \(totalMemory / 1024 / 1024 / 1024) GB
        :::: alloc               \(allocAvgTime.formattedWithPrecision()) s
        :::: access (sequential) \(accessSequentialAvgTime.formattedWithPrecision()) s
        :::: access (random)     \(accessRandomAvgTime.formattedWithPrecision()) s
        :::: access (concurrent) \(accessConcurrentAvgTime.formattedWithPrecision()) s
        ----
        :::: score               \(score.formattedWithPrecision())
        """
    }
}

public struct StorageConfig: Config {
    fileprivate typealias CStruct = AcubenchFFI.StorageConfig
    
    public let dir: URL
    
    public let accessSequentialIters: Int
    public let accessSequentialDataSizeMB: Int
    
    public let accessRandomIters: Int
    public let accessRandomDataSizeMB: Int
    
    public init(
        dir: URL,
        accessSequentialIters: Int = Self.Default.iters,
        accessSequentialDataSizeMB: Int = Self.Default.accessDataSizeMB,
        accessRandomIters: Int = Self.Default.iters,
        accessRandomDataSizeMB: Int = Self.Default.accessDataSizeMB
    ) {
        self.dir = dir
        self.accessSequentialIters = accessSequentialIters
        self.accessSequentialDataSizeMB = accessSequentialDataSizeMB
        self.accessRandomIters = accessRandomIters
        self.accessRandomDataSizeMB = accessRandomDataSizeMB
    }
    
    public init(
        accessSequentialIters: Int = Self.Default.iters,
        accessSequentialDataSizeMB: Int = Self.Default.accessDataSizeMB,
        accessRandomIters: Int = Self.Default.iters,
        accessRandomDataSizeMB: Int = Self.Default.accessDataSizeMB
    ) {
        self.init(
            dir: FileManager.default.workingDir,
            accessSequentialIters: accessSequentialIters,
            accessSequentialDataSizeMB: accessSequentialDataSizeMB,
            accessRandomIters: accessRandomIters,
            accessRandomDataSizeMB: accessRandomDataSizeMB
        )
    }
    
    public init(
        dir: URL,
        iters: Int = Self.Default.iters,
        accessDataSizeMB: Int = Self.Default.accessDataSizeMB
    ) {
        self.init(
            dir: dir,
            accessSequentialIters: iters,
            accessSequentialDataSizeMB: accessDataSizeMB,
            accessRandomIters: iters,
            accessRandomDataSizeMB: accessDataSizeMB
        )
    }
    
    public init(
        iters: Int = Self.Default.iters,
        accessDataSizeMB: Int = Self.Default.accessDataSizeMB
    ) {
        self.init(
            dir: FileManager.default.workingDir,
            accessSequentialIters: iters,
            accessSequentialDataSizeMB: accessDataSizeMB,
            accessRandomIters: iters,
            accessRandomDataSizeMB: accessDataSizeMB
        )
    }
    
    fileprivate func toCStruct() -> AcubenchFFI.StorageConfig {
        var cStruct = AcubenchFFI.StorageConfig()
        let dirData = dir.path.data(using: .utf8)!
        cStruct.dir = dirData.withUnsafeBytes { $0.baseAddress!.assumingMemoryBound(to: CChar.self) }
        cStruct.dir_len = dirData.count
        cStruct.access_seq_iters = accessSequentialIters
        cStruct.access_seq_data_len_mb = accessSequentialDataSizeMB
        cStruct.access_rand_iters = accessRandomIters
        cStruct.access_rand_data_len_mb = accessRandomDataSizeMB
        
        return cStruct
    }
    
    public struct Default {
        public static let iters: Int = 1
        public static let accessDataSizeMB: Int = 50
    }
}

public struct StorageReport: Report {
    fileprivate typealias CStruct = AcubenchFFI.StorageReport
    
    public let availableStorage: UInt
    public let accessSequentialAvgTime: Double
    public let accessRandomAvgTime: Double
    public let score: Double
    
    public let error: String?
    
    fileprivate init(from cStruct: UnsafeMutablePointer<AcubenchFFI.StorageReport>) {
        availableStorage = UInt(cStruct.pointee.avail_storage)
        accessSequentialAvgTime = cStruct.pointee.access_seq_avg_t
        accessRandomAvgTime = cStruct.pointee.access_rand_avg_t
        score = cStruct.pointee.score
        if let err = cStruct.pointee.err {
            error = String(cString: err)
        } else {
            error = nil
        }
        drop_storage_report(cStruct)
    }
}

@available(macOS 12.0, *)
extension StorageReport: CustomDebugStringConvertible {
    public var debugDescription: String {
        """
        Storage
        :::: available           \(availableStorage / 1024 / 1024 / 1024) GB
        :::: access (sequential) \(accessSequentialAvgTime.formattedWithPrecision()) s
        :::: access (random)     \(accessRandomAvgTime.formattedWithPrecision()) s
        ----
        :::: score               \(score.formattedWithPrecision())
        """
    }
}

public class Acubench {
    private let ptr: UnsafeMutableRawPointer

    public init() {
        let totalRam = Int64(ProcessInfo.processInfo.physicalMemory)
        let availableStorage = FileManager.default.availableStorage
        let hwcaps = Acubench.getHardwareCapabilities()
        let i8mmMask = TypedU64(t: 0, v: hwcaps.i8mmMask)
        let sveMask = TypedU64(t: 0, v: hwcaps.sveMask)
        ptr = new_bench(
            UInt64(totalRam),
            UInt64(availableStorage),
            hwcaps.hwcap,
            hwcaps.hwcap2,
            sveMask.toCStruct(),
            i8mmMask.toCStruct()
        )
    }
    
    public func cpu(config: CPUConfig = CPUConfig()) throws -> CPUReport {
        try run(with: config, bench_cpu)
    }
    
    public func cpuMultithread(config: CPUConfig = CPUConfig()) throws -> CPUReport {
        try run(with: config, bench_cpu_multithread)
    }
    
    public func ram(config: RAMConfig = RAMConfig()) throws -> RAMReport {
        try run(with: config, bench_ram)
    }
    
    public func storage(config: StorageConfig = StorageConfig()) throws -> StorageReport {
        try run(with: config, bench_storage)
    }
    
    private func run<C: Config, R: Report>(with config: C, _ bench: @escaping (UnsafeMutableRawPointer, C.CStruct) -> UnsafeMutablePointer<R.CStruct>?) throws -> R {
        guard let cReport = bench(ptr, config.toCStruct()) else {
            throw Error.noPointer
        }
        
        let report = R(from: cReport)
        if let error = report.error {
            throw Error.raw(error)
        }
        
        return report
    }
    
    deinit {
        drop_bench(ptr)
    }

    static func getHardwareCapabilities() -> HardwareCapabilities {
        var hwcap: UInt64 = 0
        let hwcap2: UInt64 = 0
        let sveMask: UInt64 = 0
        let i8mmMask: UInt64 = 0

        #if arch(arm64)
            var size = MemoryLayout<Int>.size
            var features: Int = 0
            if sysctlbyname("hw.optional.neon", &features, &size, nil, 0) == 0 && features != 0 {
                hwcap |= 1 << 1
            }
            var cpuSubtype: Int = 0
            size = MemoryLayout<Int>.size
            if sysctlbyname("hw.cpusubtype", &cpuSubtype, &size, nil, 0) == 0 {
                print("CPU Subtype: \(cpuSubtype)")
            }
        #endif

        return HardwareCapabilities(
            hwcap: hwcap,
            hwcap2: hwcap2,
            sveMask: sveMask,
            i8mmMask: i8mmMask
        )
    }
    
    struct HardwareCapabilities {
        let hwcap: UInt64
        let hwcap2: UInt64
        let sveMask: UInt64
        let i8mmMask: UInt64
    }
    
    enum Error: Swift.Error {
        case noPointer
        case raw(String)
    }
}

private protocol Config {
    associatedtype CStruct
    
    func toCStruct() -> CStruct
}

private protocol Report {
    associatedtype CStruct
    
    init(from cStruct: UnsafeMutablePointer<CStruct>)
    
    var score: Double { get }
    var error: String? { get }
}

extension FileManager {
    var workingDir: URL {
        guard let dir = urls(for: .documentDirectory, in: .userDomainMask).first else {
            return temporaryDirectory
        }
        return dir
    }

    var availableStorage: Int {
        do {
            let path = workingDir.path
            let attributes = try attributesOfFileSystem(forPath: path)
            guard let size = attributes[.systemFreeSize] as? NSNumber else {
                return 0
            }
            return Int(truncating: size)
        } catch {
            return 0
        }
    }
}

extension Double {
    func formattedWithPrecision(_ digits: Int = 10) -> String {
        formatted(.number.precision(.fractionLength(digits)))
    }
}
