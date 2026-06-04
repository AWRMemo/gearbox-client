import Flutter
import UIKit
import XCTest

@testable import Runner

class ModelDownloadPluginTests: XCTestCase {

    var plugin: ModelDownloadPlugin!

    override func setUp() {
        super.setUp()
        plugin = ModelDownloadPlugin()
    }

    func testResumeDataViaCancelMethodChannel() {
        let expectation = XCTestExpectation(description: "cancelDownload completes")
        let call = FlutterMethodCall(methodName: "cancelDownload", arguments: nil)
        plugin.handle(call) { result in
            XCTAssertEqual(result as? Bool, true)
            // After cancellation, resume data may or may not be present,
            // but the call must succeed gracefully.
            expectation.fulfill()
        }
        wait(for: [expectation], timeout: 1.0)
    }

    func testVerifyIntegrityViaMethodChannel() {
        let expectation = XCTestExpectation(description: "verifyIntegrity returns false for missing file")
        let args: [String: Any] = ["path": "/nonexistent/path.bin", "sha256": "abc"]
        let call = FlutterMethodCall(methodName: "verifyIntegrity", arguments: args)
        plugin.handle(call) { result in
            XCTAssertEqual(result as? Bool, false)
            expectation.fulfill()
        }
        wait(for: [expectation], timeout: 1.0)
    }
}
