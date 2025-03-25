//
//  AcuBenchmarkTests.swift
//  AcurastBenchmark
//
//  Created by Pablo Martinez Piles on 6/3/25.
//

import Testing
@testable import AcurastBenchmark

@Test func test_cpu() async throws {
    let acuBenchmark = Acubench()
    let cpu = acuBenchmark.cpu()
    assert(cpu.cryptoErr == nil)
}
