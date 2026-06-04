import Flutter
import UIKit
import CryptoKit

/// Plugin that handles model downloads on iOS using URLSession for resumable
/// background downloads.  Communicates with Flutter via MethodChannel and
/// EventChannel for progress.
public class ModelDownloadPlugin: NSObject, FlutterPlugin, FlutterStreamHandler {
    private var eventSink: FlutterEventSink?
    private var downloadTask: URLSessionDownloadTask?
    private var resumeData: Data?
    private var currentDestination: URL?
    private var session: URLSession?

    // MARK: - FlutterPlugin

    public static func register(with registrar: FlutterPluginRegistrar) {
        let methodChannel = FlutterMethodChannel(
            name: "com.gearbox.model_download",
            binaryMessenger: registrar.messenger()
        )
        let eventChannel = FlutterEventChannel(
            name: "com.gearbox.model_download/events",
            binaryMessenger: registrar.messenger()
        )
        let instance = ModelDownloadPlugin()
        registrar.addMethodCallDelegate(instance, channel: methodChannel)
        eventChannel.setStreamHandler(instance)
    }

    public func handle(_ call: FlutterMethodCall, result: @escaping FlutterResult) {
        switch call.method {
        case "startDownload":
            guard let args = call.arguments as? [String: Any],
                  let urlString = args["url"] as? String,
                  let savePath = args["savePath"] as? String else {
                result(FlutterError(code: "INVALID_ARGUMENT",
                                    message: "Missing url or savePath",
                                    details: nil))
                return
            }
            startDownload(urlString: urlString, savePath: savePath, result: result)
        case "cancelDownload":
            cancelDownload(result: result)
        case "verifyIntegrity":
            guard let args = call.arguments as? [String: Any],
                  let path = args["path"] as? String,
                  let expected = args["sha256"] as? String else {
                result(FlutterError(code: "INVALID_ARGUMENT",
                                    message: "Missing path or sha256",
                                    details: nil))
                return
            }
            verifyIntegrity(path: path, expected: expected, result: result)
        default:
            result(FlutterMethodNotImplemented)
        }
    }

    // MARK: - FlutterStreamHandler

    public func onListen(withArguments arguments: Any?, eventSink events: @escaping FlutterEventSink) -> FlutterError? {
        self.eventSink = events
        return nil
    }

    public func onCancel(withArguments arguments: Any?) -> FlutterError? {
        self.eventSink = nil
        return nil
    }

    // MARK: - Download logic

    private func startDownload(urlString: String, savePath: String, result: @escaping FlutterResult) {
        currentDestination = URL(fileURLWithPath: savePath)
        guard let url = URL(string: urlString) else {
            result(FlutterError(code: "INVALID_URL", message: "Bad URL: \(urlString)", details: nil))
            return
        }
        let config = URLSessionConfiguration.default
        config.isDiscretionary = false
        session = URLSession(configuration: config, delegate: self, delegateQueue: nil)
        if let data = resumeData {
            downloadTask = session?.downloadTask(withResumeData: data)
        } else {
            downloadTask = session?.downloadTask(with: url)
        }
        downloadTask?.resume()
        result(true)
    }

    private func cancelDownload(result: @escaping FlutterResult) {
        downloadTask?.cancel { data in
            self.resumeData = data
        }
        result(true)
    }

    private func verifyIntegrity(path: String, expected: String, result: @escaping FlutterResult) {
        DispatchQueue.global(qos: .utility).async {
            guard let data = try? Data(contentsOf: URL(fileURLWithPath: path)) else {
                DispatchQueue.main.async { result(false) }
                return
            }
            let digest = SHA256.hash(data: data)
            let hex = digest.map { String(format: "%02hhx", $0) }.joined()
            DispatchQueue.main.async { result(hex.lowercased() == expected.lowercased()) }
        }
    }
}

// MARK: - URLSessionDownloadDelegate

extension ModelDownloadPlugin: URLSessionDownloadDelegate {
    public func urlSession(_ session: URLSession,
                           downloadTask: URLSessionDownloadTask,
                           didWriteData bytesWritten: Int64,
                           totalBytesWritten: Int64,
                           totalBytesExpectedToWrite: Int64) {
        let progress: [String: Any] = [
            "bytesDownloaded": totalBytesWritten,
            "totalBytes": totalBytesExpectedToWrite,
            "status": "downloading"
        ]
        DispatchQueue.main.async {
            self.eventSink?(progress)
        }
    }

    public func urlSession(_ session: URLSession,
                           downloadTask: URLSessionDownloadTask,
                           didFinishDownloadingTo location: URL) {
        guard let destination = currentDestination else { return }
        try? FileManager.default.removeItem(at: destination)
        try? FileManager.default.moveItem(at: location, to: destination)
        // Exclude from iCloud backup
        var resourceValues = URLResourceValues()
        resourceValues.isExcludedFromBackup = true
        try? destination.setResourceValues(resourceValues)
        DispatchQueue.main.async {
            self.eventSink?([
                "bytesDownloaded": 1,
                "totalBytes": 1,
                "status": "completed"
            ])
        }
    }

    public func urlSession(_ session: URLSession,
                           task: URLSessionTask,
                           didCompleteWithError error: Error?) {
        if let error = error {
            DispatchQueue.main.async {
                // Attempt to capture resume data
                var resumeData: Data?
                if let userInfo = (error as? URLError)?.userInfo {
                    resumeData = userInfo[NSURLSessionDownloadTaskResumeData] as? Data
                }
                if let data = resumeData {
                    self.resumeData = data
                }
                self.eventSink?([
                    "bytesDownloaded": 0,
                    "totalBytes": 0,
                    "status": "failed",
                    "errorMessage": error.localizedDescription
                ])
            }
        }
    }
}
