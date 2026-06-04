import Flutter
import UIKit
import MlxBridge

/// iOS-native AI enrichment plugin using MLX Swift via MlxBridgePackage.
///
/// Communicates with Flutter via MethodChannel `com.gearbox.ai`.
/// Delegates inference to `MlxBridge.enrichHighlight`.
public class AiPlugin: NSObject, FlutterPlugin {
    private let modelQueue = DispatchQueue(label: "com.gearbox.ai", qos: .userInitiated)

    // MARK: - FlutterPlugin

    public static func register(with registrar: FlutterPluginRegistrar) {
        let channel = FlutterMethodChannel(name: "com.gearbox.ai",
                                           binaryMessenger: registrar.messenger())
        let instance = AiPlugin()
        registrar.addMethodCallDelegate(instance, channel: channel)
    }

    public func handle(_ call: FlutterMethodCall, result: @escaping FlutterResult) {
        switch call.method {
        case "enrichHighlight":
            guard let args = call.arguments as? [String: Any],
                  let text = args["text"] as? String,
                  !text.isEmpty else {
                result(FlutterError(code: "INVALID_ARGUMENT",
                                    message: "Missing or empty 'text'",
                                    details: nil))
                return
            }
            modelQueue.async {
                Task {
                    do {
                        let json = try await MlxBridge.enrichHighlight(text: text)
                        DispatchQueue.main.async {
                            result(json)
                        }
                    } catch {
                        DispatchQueue.main.async {
                            result(FlutterError(code: "INFERENCE_ERROR",
                                                message: error.localizedDescription,
                                                details: nil))
                        }
                    }
                }
            }
        default:
            result(FlutterMethodNotImplemented)
        }
    }

    /// Called by the app delegate on memory warning.
    public func releaseModel() {
        MlxBridge.releaseModel()
    }
}
