//
//  AcuBenchmark.swift
//  AcurastProcessor
//
//  Created by Pablo Martinez Piles on 6/3/25.
//

import Foundation

private let KB: Int64 = 1024
private let MB: Int64 = KB * 1024

@_silgen_name("new_bench") fileprivate func new_bench(total_ram: UInt64, avail_storage: UInt64, hwcap: UInt64, hwcap2: UInt64, sve_mask: TypedU64, i8mm_mask: TypedU64) -> UnsafeMutableRawPointer
@_silgen_name("drop_bench") fileprivate func drop_bench(ptr: UnsafeMutableRawPointer)
@_silgen_name("bench_cpu") fileprivate func bench_cpu(ptr: UnsafeMutableRawPointer, config: CpuConfig) -> UnsafeMutablePointer<CpuReport>
@_silgen_name("bench_cpu_multithread") fileprivate func bench_cpu_multithread(ptr: UnsafeMutableRawPointer, config: CpuConfig) -> UnsafeMutablePointer<CpuReport>
@_silgen_name("drop_cpu_report") fileprivate func drop_cpu_report(ptr: UnsafeMutablePointer<CpuReport>)
@_silgen_name("bench_ram") fileprivate func bench_ram(ptr: UnsafeMutableRawPointer, config: RamConfig) -> UnsafeMutablePointer<RamReport>
@_silgen_name("drop_ram_report") fileprivate func drop_ram_report(ptr: UnsafeMutablePointer<RamReport>)
@_silgen_name("bench_storage") fileprivate func bench_storage(ptr: UnsafeMutableRawPointer, config: StorageConfig) -> UnsafeMutablePointer<StorageReport>
@_silgen_name("drop_storage_report") fileprivate func drop_storage_report(ptr: UnsafeMutablePointer<StorageReport>)

public struct TypedU64 {
    public let t: UInt8
    public let v: UInt64
    
    public init(t: UInt8, v: UInt64) {
        self.t = t
        self.v = v
    }
}

public struct CpuConfig {
    public let cryptoDuration: Int
    public let cryptoDataLen: Int
    
    public let mathDuration: Int
    public let mathDataLen: Int
    public let mathSIMD: Bool
    
    public let sortDuration: Int
    public let sortDataLen: Int
    
    public init(
        cryptoDuration: TimeInterval = CpuConfig.durationDefault,
        cryptoDataSize: Int64 = CpuConfig.cryptoDataSizeDefault,
        mathDuration: TimeInterval = CpuConfig.durationDefault,
        mathDataSize: Int64 = CpuConfig.mathDataSizeDefault,
        mathSIMD: Bool = CpuConfig.mathSIMDDefault,
        sortDuration: TimeInterval = CpuConfig.durationDefault,
        sortDataSize: Int64 = CpuConfig.sortDataSizeDefault
    ) {
        self.cryptoDuration = Int(cryptoDuration * 1000)
        self.cryptoDataLen = Int(cryptoDataSize)
        self.mathDuration = Int(mathDuration * 1000)
        self.mathDataLen = Int(mathDataSize)
        self.mathSIMD = mathSIMD
        self.sortDuration = Int(sortDuration * 1000)
        self.sortDataLen = Int(sortDataSize)
    }
    
    public init(
        duration: TimeInterval = CpuConfig.durationDefault,
        cryptoDataSize: Int64 = CpuConfig.cryptoDataSizeDefault,
        mathDataSize: Int64 = CpuConfig.mathDataSizeDefault,
        sortDataSize: Int64 = CpuConfig.sortDataSizeDefault
    ) {
        self.init(
            cryptoDuration: duration,
            cryptoDataSize: cryptoDataSize,
            mathDuration: duration,
            mathDataSize: mathDataSize,
            sortDuration: duration,
            sortDataSize: sortDataSize
        )
    }
    
    public static let durationDefault: TimeInterval = 1.0
    public static let cryptoDataSizeDefault: Int64 = 10 * KB
    public static let mathDataSizeDefault: Int64 = 200
    public static let mathSIMDDefault: Bool = true
    public static let sortDataSizeDefault: Int64 = 100_000
}

public struct CpuReport {
    public let cryptoTps: Double
    public let mathTps: Double
    public let sortTps: Double
    public let score: Double
    
    public let err: String?
    
    init(cStruct: UnsafeMutablePointer<CpuReport>) {
        self.cryptoTps = cStruct.pointee.cryptoTps
        self.mathTps = cStruct.pointee.mathTps
        self.sortTps = cStruct.pointee.sortTps
        self.score = cStruct.pointee.score
        self.err = cStruct.pointee.err
        drop_cpu_report(ptr: cStruct)
    }
}

@available(iOS 15.0, *)
extension CpuReport: CustomDebugStringConvertible {
    public var debugDescription: String {
        """
        CPU
        :::: crypto \(cryptoTps.formatted()) ops/s
        :::: math   \(mathTps.formatted()) ops/s
        :::: sort   \(sortTps.formatted()) ops/s
        ----
        :::: score  \(score.formatted())
        """
    }
}

public struct RamConfig {
    public let allocIters: Int
    public let allocDataLen: Int
    
    public let accessSeqIters: Int
    public let accessSeqDataLen: Int
    
    public let accessRandIters: Int
    public let accessRandDataLen: Int
    
    public let accessConcurrIters: Int
    public let accessConcurrDataLen: Int
    
    public init(
        allocIters: Int64 = RamConfig.itersDefault,
        allocDataSize: Int64 = RamConfig.allocDataSizeDefault,
        accessSequentialIters: Int64 = RamConfig.itersDefault,
        accessSequentialDataSize: Int64 = RamConfig.accessDataSizeDefault,
        accessRandomIters: Int64 = RamConfig.itersDefault,
        accessRandomDataSize: Int64 = RamConfig.accessDataSizeDefault,
        accessConcurrentIters: Int64 = RamConfig.itersDefault,
        accessConcurrentDataSize: Int64 = RamConfig.accessDataSizeDefault
    ) {
        self.allocIters = Int(allocIters)
        self.allocDataLen = Int(allocDataSize)
        self.accessSeqIters = Int(accessSequentialIters)
        self.accessSeqDataLen = Int(accessSequentialDataSize)
        self.accessRandIters = Int(accessRandomIters)
        self.accessRandDataLen = Int(accessRandomDataSize)
        self.accessConcurrIters = Int(accessConcurrentIters)
        self.accessConcurrDataLen = Int(accessConcurrentDataSize)
    }
    
    public init(
        allocIters: Int64 = RamConfig.itersDefault,
        allocDataSize: Int64 = RamConfig.allocDataSizeDefault,
        accessIters: Int64 = RamConfig.itersDefault,
        accessDataSize: Int64 = RamConfig.accessDataSizeDefault
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
        iters: Int64 = RamConfig.itersDefault,
        allocDataSize: Int64 = RamConfig.allocDataSizeDefault,
        accessDataSize: Int64 = RamConfig.accessDataSizeDefault
    ) {
        self.init(
            allocIters: iters,
            allocDataSize: allocDataSize,
            accessIters: iters,
            accessDataSize: accessDataSize
        )
    }
    
    public static let allocDataSizeDefault: Int64 = 64 * MB
    public static let accessDataSizeDefault: Int64 = 64 * KB
    public static let itersDefault: Int64 = 10
}

public struct RamReport {
    public let totalMemory: UInt64
    public let allocAvgTime: Double
    public let accessSeqAvgTime: Double
    public let accessRandAvgTime: Double
    public let accessConAvgTime: Double
    public let score: Double
    
    public let err: String?
    
    init(cStruct: UnsafeMutablePointer<RamReport>) {
        self.totalMemory = cStruct.pointee.totalMemory
        self.allocAvgTime = cStruct.pointee.allocAvgTime
        self.accessSeqAvgTime = cStruct.pointee.accessSeqAvgTime
        self.accessRandAvgTime = cStruct.pointee.accessRandAvgTime
        self.accessConAvgTime = cStruct.pointee.accessSeqAvgTime
        self.score = cStruct.pointee.score
        self.err = cStruct.pointee.err
        drop_ram_report(ptr: cStruct)
    }
}

@available(iOS 15.0, *)
extension RamReport: CustomDebugStringConvertible {
    public var debugDescription: String {
        """
        RAM
        :::: total memory        \(totalMemory / 1024 / 1024) GB
        :::: alloc               \(allocAvgTime.formatted()) s
        :::: access (sequential) \(accessSeqAvgTime.formatted()) s
        :::: access (random)     \(accessRandAvgTime.formatted()) s
        :::: access (concurrent) \(accessConAvgTime.formatted()) s
        ----
        :::: score               \(score.formatted())
        """
    }
}

public struct StorageConfig {
    public let dir: UnsafePointer<UInt8>
    public let dirLen: Int
    
    public let accessSeqIters: Int
    public let accessSeqDataLenMB: Int
    
    public let accessRandIters: Int
    public let accessRandDataLenMB: Int
    
    public init(
        dir: URL,
        accessSequentialIters: Int64 = StorageConfig.itersDefault,
        accessSequentialDataSizeMB: Int64 = StorageConfig.accessDataSizeMBDefault,
        accessRandomIters: Int64 = StorageConfig.itersDefault,
        accessRandomDataSizeMB: Int64 = StorageConfig.accessDataSizeMBDefault
    ) {
        let dirData = dir.path.data(using: .utf8)!
        self.dir = dirData.withUnsafeBytes { $0.baseAddress!.assumingMemoryBound(to: UInt8.self) }
        self.dirLen = dirData.count
        self.accessSeqIters = Int(accessSequentialIters)
        self.accessSeqDataLenMB = Int(accessSequentialDataSizeMB)
        self.accessRandIters = Int(accessRandomIters)
        self.accessRandDataLenMB = Int(accessRandomDataSizeMB)
    }
    
    public init(
        accessSequentialIters: Int64 = StorageConfig.itersDefault,
        accessSequentialDataSizeMB: Int64 = StorageConfig.accessDataSizeMBDefault,
        accessRandomIters: Int64 = StorageConfig.itersDefault,
        accessRandomDataSizeMB: Int64 = StorageConfig.accessDataSizeMBDefault
    ) {
        self.init(
            dir: FileManager.default.temporaryDirectory,
            accessSequentialIters: accessSequentialIters,
            accessSequentialDataSizeMB: accessSequentialDataSizeMB,
            accessRandomIters: accessRandomIters,
            accessRandomDataSizeMB: accessRandomDataSizeMB
        )
    }
    
    public init(
        dir: URL,
        iters: Int64 = StorageConfig.itersDefault,
        accessDataSizeMB: Int64 = StorageConfig.accessDataSizeMBDefault
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
        iters: Int64 = StorageConfig.itersDefault,
        accessDataSizeMB: Int64 = StorageConfig.accessDataSizeMBDefault
    ) {
        self.init(
            dir: FileManager.default.temporaryDirectory,
            accessSequentialIters: iters,
            accessSequentialDataSizeMB: accessDataSizeMB,
            accessRandomIters: iters,
            accessRandomDataSizeMB: accessDataSizeMB
        )
    }
    
    public static let accessDataSizeMBDefault: Int64 = 50
    public static let itersDefault: Int64 = 1
}

public struct StorageReport {
    public let availableStorage: UInt64
    public let accessSeqAvgTime: Double
    public let accessRandAvgTime: Double
    public let score: Double
    public let err: String?
    
    init(cStruct: UnsafeMutablePointer<StorageReport>) {
        self.availableStorage = cStruct.pointee.availableStorage
        self.accessSeqAvgTime = cStruct.pointee.accessSeqAvgTime
        self.accessRandAvgTime = cStruct.pointee.accessRandAvgTime
        self.score = cStruct.pointee.score
        self.err = cStruct.pointee.err
        drop_storage_report(ptr: cStruct)
    }
}

@available(iOS 15.0, *)
extension StorageReport: CustomDebugStringConvertible {
    public var debugDescription: String {
        """
        Storage
        :::: available           \(availableStorage / 1024 / 1024) GB
        :::: access (sequential) \(accessSeqAvgTime.formatted()) s
        :::: access (random)     \(accessRandAvgTime.formatted()) s
        ----
        :::: score               \(score.formatted())
        """
    }
}

public class Acubench {
    private let ptr: UnsafeMutableRawPointer
    
    public init() {
        let totalRam = Int64(ProcessInfo.processInfo.physicalMemory)
        let availStorage = FileManager.default.availableStorage
        let hardwareCapabilities = Acubench.getHardwareCapabilities()
        let i8mmMask = TypedU64(t: 0, v: hardwareCapabilities.i8mmMask)
        let sveMask = TypedU64(t: 0, v: hardwareCapabilities.sveMask)
        ptr = new_bench(
            total_ram: UInt64(totalRam),
            avail_storage: UInt64(availStorage),
            hwcap: hardwareCapabilities.hwcap,
            hwcap2: hardwareCapabilities.hwcap2,
            sve_mask: sveMask,
            i8mm_mask: i8mmMask
        )
    }
    
    public func cpu(config: CpuConfig = CpuConfig()) -> CpuReport {
        return CpuReport(cStruct: bench_cpu(ptr: ptr, config: config))
    }
    
    public func cpuMultithread(config: CpuConfig = CpuConfig()) -> CpuReport {
        return CpuReport(cStruct: bench_cpu_multithread(ptr: ptr, config: config))
    }
    
    public func ram(config: RamConfig = RamConfig()) -> RamReport {
        return RamReport(cStruct: bench_ram(ptr: ptr, config: config))
    }
    
    public func storage(config: StorageConfig) -> StorageReport {
        return StorageReport(cStruct: bench_storage(ptr: ptr, config: config))
    }
    
    public func storage() -> StorageReport {
        return storage(config: StorageConfig())
    }
    
    public func destroy() {
        drop_bench(ptr: ptr)
    }
    
    deinit {
        destroy()
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
}

extension FileManager {
    var availableStorage: Int64 {
        do {
            let attributes = try attributesOfItem(atPath: FileManager.default.temporaryDirectory.path)
            return attributes[.systemFreeSize] as? Int64 ?? 0
        } catch {
            print("Failed to get available storage: \(error)")
            return 0
        }
    }
}


