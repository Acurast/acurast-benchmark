//
//  AcubenchTests.swift
//  Acubench
//
//  Created by Pablo Martinez Piles on 6/3/25.
//

import Testing
@testable import Acubench

@Test func test_all() async throws {
    let acubench = Acubench()
    
    let cpuReport = try acubench.cpu()
    #expect(cpuReport.cryptoTPS > 0)
    #expect(cpuReport.mathTPS > 0)
    #expect(cpuReport.sortTPS > 0)
    #expect(cpuReport.score == cpuReport.expectedScore)
    
    let cpuMultithreadReport = try acubench.cpuMultithread()
    #expect(cpuMultithreadReport.cryptoTPS > 0)
    #expect(cpuMultithreadReport.mathTPS > 0)
    #expect(cpuMultithreadReport.sortTPS > 0)
    #expect(cpuMultithreadReport.score == cpuMultithreadReport.expectedScore)
    
    let ramReport = try acubench.ram()
    #expect(ramReport.totalMemory > 0)
    #expect(ramReport.allocAvgTime > 0)
    #expect(ramReport.accessSequentialAvgTime > 0)
    #expect(ramReport.accessRandomAvgTime > 0)
    #expect(ramReport.accessConcurrentAvgTime > 0)
    #expect(ramReport.score == ramReport.expectedScore)
    
    let storageReport = try acubench.storage()
    #expect(storageReport.availableStorage > 0)
    #expect(storageReport.accessSequentialAvgTime > 0)
    #expect(storageReport.accessRandomAvgTime > 0)
    #expect(storageReport.score == storageReport.expectedScore)
    
    print(cpuReport)
    print(cpuMultithreadReport)
    print(ramReport)
    print(storageReport)
}

@Test func test_cpu() async throws {
    let acubench = Acubench()
    let report = try acubench.cpu()
    
    #expect(report.cryptoTPS > 0)
    #expect(report.mathTPS > 0)
    #expect(report.sortTPS > 0)
    #expect(report.score == report.expectedScore)
    
    print(report)
}

@Test func test_cpuMultithread() async throws {
    let acubench = Acubench()
    let report = try acubench.cpuMultithread()
    
    #expect(report.cryptoTPS > 0)
    #expect(report.mathTPS > 0)
    #expect(report.sortTPS > 0)
    #expect(report.score == report.expectedScore)
    
    print(report)
}

@Test func test_ram() async throws {
    let acubench = Acubench()
    let report = try acubench.ram()
    
    #expect(report.totalMemory > 0)
    #expect(report.allocAvgTime > 0)
    #expect(report.accessSequentialAvgTime > 0)
    #expect(report.accessRandomAvgTime > 0)
    #expect(report.accessConcurrentAvgTime > 0)
    #expect(report.score == report.expectedScore)
    
    print(report)
}

@Test func test_storage() async throws {
    let acubench = Acubench()
    let report = try acubench.storage()
    
    #expect(report.availableStorage > 0)
    #expect(report.accessSequentialAvgTime > 0)
    #expect(report.accessRandomAvgTime > 0)
    #expect(report.score == report.expectedScore)
    
    print(report)
}


private extension CPUReport {
    var expectedScore: Double {
        (cryptoTPS + mathTPS + sortTPS) / 3.0
    }
}

private extension RAMReport {
    var expectedScore: Double {
        (allocAvgTime.inv() + accessSequentialAvgTime.inv() + accessRandomAvgTime.inv() + accessConcurrentAvgTime.inv()) / 4.0
    }
}

private extension StorageReport {
    var expectedScore: Double {
        (accessSequentialAvgTime.inv() + accessRandomAvgTime.inv()) / 2.0
    }
}

private extension Double {
    func inv() -> Double {
        guard self != 0 else {
            return 0
        }
        
        return 1.0 / self
    }
}
