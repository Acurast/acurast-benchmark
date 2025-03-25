//
//  AcuBenchmark.swift
//  AcurastProcessor
//
//  Created by Pablo Martinez Piles on 6/3/25.
//

import Foundation
import UIKit

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
    public let sortDuration: Int
    public let sortDataLen: Int
    
    public init(
        cryptoDuration: TimeInterval = CpuConfig.DURATION_DEFAULT,
        cryptoDataSize: Int64 = CpuConfig.CRYPTO_DATA_SIZE_DEFAULT,
        mathDuration: TimeInterval = CpuConfig.DURATION_DEFAULT,
        mathDataSize: Int64 = CpuConfig.MATH_DATA_SIZE_DEFAULT,
        sortDuration: TimeInterval = CpuConfig.DURATION_DEFAULT,
        sortDataSize: Int64 = CpuConfig.SORT_DATA_SIZE_DEFAULT
    ) {
        self.cryptoDuration = Int(cryptoDuration * 1000)
        self.cryptoDataLen = Int(cryptoDataSize)
        self.mathDuration = Int(mathDuration * 1000)
        self.mathDataLen = Int(mathDataSize)
        self.sortDuration = Int(sortDuration * 1000)
        self.sortDataLen = Int(sortDataSize)
    }
    
    public init(
        duration: TimeInterval = CpuConfig.DURATION_DEFAULT,
        cryptoDataSize: Int64 = CpuConfig.CRYPTO_DATA_SIZE_DEFAULT,
        mathDataSize: Int64 = CpuConfig.MATH_DATA_SIZE_DEFAULT,
        sortDataSize: Int64 = CpuConfig.SORT_DATA_SIZE_DEFAULT
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
    
    public static let DURATION_DEFAULT: TimeInterval = 1.0
    public static let CRYPTO_DATA_SIZE_DEFAULT: Int64 = 10 * KB
    public static let MATH_DATA_SIZE_DEFAULT: Int64 = 200
    public static let SORT_DATA_SIZE_DEFAULT: Int64 = 100_000
}

public struct CpuReport {
    public let cryptoTps: Double
    public let cryptoErr: String?
    public let mathTps: Double
    public let mathErr: String?
    public let sortTps: Double
    public let sortErr: String?
    
    init(cStruct: UnsafeMutablePointer<CpuReport>) {
        self.cryptoTps = cStruct.pointee.cryptoTps
        self.cryptoErr = cStruct.pointee.cryptoErr != nil ? String(cString: cStruct.pointee.cryptoErr!) : nil
        self.mathTps = cStruct.pointee.mathTps
        self.mathErr = cStruct.pointee.mathErr != nil ? String(cString: cStruct.pointee.mathErr!) : nil
        self.sortTps = cStruct.pointee.sortTps
        self.sortErr = cStruct.pointee.sortErr != nil ? String(cString: cStruct.pointee.sortErr!) : nil
        drop_cpu_report(ptr: cStruct)
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
        allocIters: Int64 = RamConfig.ITERS_DEFAULT,
        allocDataSize: Int64 = RamConfig.ALLOC_DATA_SIZE_DEFAULT,
        accessSequentialIters: Int64 = RamConfig.ITERS_DEFAULT,
        accessSequentialDataSize: Int64 = RamConfig.ACCESS_DATA_SIZE_DEFAULT,
        accessRandomIters: Int64 = RamConfig.ITERS_DEFAULT,
        accessRandomDataSize: Int64 = RamConfig.ACCESS_DATA_SIZE_DEFAULT,
        accessConcurrentIters: Int64 = RamConfig.ITERS_DEFAULT,
        accessConcurrentDataSize: Int64 = RamConfig.ACCESS_DATA_SIZE_DEFAULT
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
        allocIters: Int64 = RamConfig.ITERS_DEFAULT,
        allocDataSize: Int64 = RamConfig.ALLOC_DATA_SIZE_DEFAULT,
        accessIters: Int64 = RamConfig.ITERS_DEFAULT,
        accessDataSize: Int64 = RamConfig.ACCESS_DATA_SIZE_DEFAULT
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
        iters: Int64 = RamConfig.ITERS_DEFAULT,
        allocDataSize: Int64 = RamConfig.ALLOC_DATA_SIZE_DEFAULT,
        accessDataSize: Int64 = RamConfig.ACCESS_DATA_SIZE_DEFAULT
    ) {
        self.init(
            allocIters: iters,
            allocDataSize: allocDataSize,
            accessIters: iters,
            accessDataSize: accessDataSize
        )
    }
    
    public static let ALLOC_DATA_SIZE_DEFAULT: Int64 = 64 * MB
    public static let ACCESS_DATA_SIZE_DEFAULT: Int64 = 64 * KB
    public static let ITERS_DEFAULT: Int64 = 10
}

public struct RamReport {
    public let totalMemory: UInt64
    public let allocAvgTime: Double
    public let allocErr: String?
    public let accessSeqAvgTime: Double
    public let accessRandAvgTime: Double
    public let accessConAvgTime: Double
    public let accessErr: String?
    
    init(cStruct: UnsafeMutablePointer<RamReport>) {
        self.totalMemory = cStruct.pointee.totalMemory
        self.allocAvgTime = cStruct.pointee.allocAvgTime
        self.allocErr = cStruct.pointee.allocErr != nil ? String(cString: cStruct.pointee.allocErr!) : nil
        self.accessSeqAvgTime = cStruct.pointee.accessSeqAvgTime
        self.accessRandAvgTime = cStruct.pointee.accessRandAvgTime
        self.accessConAvgTime = cStruct.pointee.accessSeqAvgTime
        self.accessErr = cStruct.pointee.accessErr != nil ? String(cString: cStruct.pointee.accessErr!) : nil
        drop_ram_report(ptr: cStruct)
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
        accessSequentialIters: Int64 = StorageConfig.ITERS_DEFAULT,
        accessSequentialDataSizeMB: Int64 = StorageConfig.ACCESS_DATA_SIZE_MB_DEFAULT,
        accessRandomIters: Int64 = StorageConfig.ITERS_DEFAULT,
        accessRandomDataSizeMB: Int64 = StorageConfig.ACCESS_DATA_SIZE_MB_DEFAULT
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
        accessSequentialIters: Int64 = StorageConfig.ITERS_DEFAULT,
        accessSequentialDataSizeMB: Int64 = StorageConfig.ACCESS_DATA_SIZE_MB_DEFAULT,
        accessRandomIters: Int64 = StorageConfig.ITERS_DEFAULT,
        accessRandomDataSizeMB: Int64 = StorageConfig.ACCESS_DATA_SIZE_MB_DEFAULT
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
        iters: Int64 = StorageConfig.ITERS_DEFAULT,
        accessDataSizeMB: Int64 = StorageConfig.ACCESS_DATA_SIZE_MB_DEFAULT
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
        iters: Int64 = StorageConfig.ITERS_DEFAULT,
        accessDataSizeMB: Int64 = StorageConfig.ACCESS_DATA_SIZE_MB_DEFAULT
    ) {
        self.init(
            dir: FileManager.default.temporaryDirectory,
            accessSequentialIters: iters,
            accessSequentialDataSizeMB: accessDataSizeMB,
            accessRandomIters: iters,
            accessRandomDataSizeMB: accessDataSizeMB
        )
    }
    
    public static let ACCESS_DATA_SIZE_MB_DEFAULT: Int64 = 50
    public static let ITERS_DEFAULT: Int64 = 1
}

public struct StorageReport {
    public let availableStorage: UInt64
    public let accessSeqAvgTime: Double
    public let accessRandAvgTime: Double
    public let accessErr: String?
    
    init(cStruct: UnsafeMutablePointer<StorageReport>) {
        self.availableStorage = cStruct.pointee.availableStorage
        self.accessSeqAvgTime = cStruct.pointee.accessSeqAvgTime
        self.accessRandAvgTime = cStruct.pointee.accessRandAvgTime
        self.accessErr = cStruct.pointee.accessErr != nil ? String(cString: cStruct.pointee.accessErr!) : nil
        drop_storage_report(ptr: cStruct)
    }
}

public class Acubench {
    private let ptr: UnsafeMutableRawPointer
    
    public init() {
        let totalRam = UIDevice.current.totalRam ?? 0
        let availStorage = FileManager.default.availableStorage
        let hardwareCapabilities = Acubench.getHardwareCapabilities()
        let i8mmMask = TypedU64(t: 0, v: hardwareCapabilities.i8mmMask)
        let sveMask = TypedU64(t: 0, v: hardwareCapabilities.sveMask)
        ptr = new_bench(total_ram: UInt64(totalRam), avail_storage: UInt64(availStorage), hwcap: hardwareCapabilities.hwcap, hwcap2: hardwareCapabilities.hwcap2, sve_mask: sveMask, i8mm_mask: i8mmMask)
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
        var hwcap2: UInt64 = 0
        var sveMask: UInt64 = 0
        var i8mmMask: UInt64 = 0

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

extension UIDevice {
    var totalRam: Int64? {
        return Int64(ProcessInfo.processInfo.physicalMemory)
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


