//
//  plugin.swift
//  Acubench
//
//  Created by Julia Samól on 08.05.2025.
//

import PackagePlugin
import Foundation

private let varHome: String = "HOME"
private let varPath: String = "PATH"

@main
struct RustBuilder: BuildToolPlugin {
    func createBuildCommands(context: PluginContext, target: Target) throws -> [Command] {
        let env = ProcessInfo.processInfo.environment
        
        let homeDir = env[.home] ?? ""
        let rustDir = context.package.directoryURL.deletingLastPathComponent().appending(path: "rust").relativePath
        let targetDir = context.pluginWorkDirectoryURL.appending(path: "target").relativePath
        
        let makeTool = try context.tool(named: "make")
        let cargoBinPath = ".cargo/bin"
        let rustupPath = homeDir.isEmpty ? "rustup" : "\(homeDir)/\(cargoBinPath)/rustup"
        let cargoPath = homeDir.isEmpty ? "cargo" : "\(homeDir)/\(cargoBinPath)/cargo"
        
        return [
            .prebuildCommand(
                displayName: "Building Rust-based xcframework using Makefile",
                executable: makeTool.url,
                arguments: [
                    "-C", context.package.directoryURL.relativePath,
                    "build",
                    "RUSTUP=\(rustupPath)",
                    "CARGO=\(cargoPath)",
                    "CARGO_MANIFEST_DIR=\(rustDir)", "CARGO_TARGET_DIR=\(targetDir)",
                    "OUTPUT_DIR=\(context.pluginWorkDirectoryURL.relativePath)"
                ],
                environment: env.filterKeys([.path]),
                outputFilesDirectory: context.pluginWorkDirectoryURL
            )
        ]
    }
}

private enum Var: String {
    case home = "HOME"
    case path = "PATH"
}

private extension Dictionary where Key == String, Value == String {
    subscript (varName: Var) -> String? {
        self[varName.rawValue]
    }
    
    func filterKeys(_ keys: Set<Var>) -> [Key:Value] {
        filter {
            guard let key = Var(rawValue: $0.key) else {
                return false
            }
            
            return keys.contains(key)
        }
    }
}
