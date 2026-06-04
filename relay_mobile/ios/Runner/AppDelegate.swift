import Flutter
import UIKit
import BackgroundTasks

@main
@objc class AppDelegate: FlutterAppDelegate, FlutterImplicitEngineDelegate {
    private var aiPlugin: AiPlugin?

    override func application(
        _ application: UIApplication,
        didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]?
    ) -> Bool {
        BackgroundSyncManager.register()
        BackgroundSyncManager.schedule()
        return super.application(application, didFinishLaunchingWithOptions: launchOptions)
    }

    override func applicationDidEnterBackground(_ application: UIApplication) {
        BackgroundSyncManager.schedule()
    }

    func didInitializeImplicitFlutterEngine(_ engineBridge: FlutterImplicitEngineBridge) {
        GeneratedPluginRegistrant.register(with: engineBridge.pluginRegistry)

        if let registrar = engineBridge.pluginRegistry.registrar(forPlugin: "AiPlugin") {
            let plugin = AiPlugin()
            plugin.register(with: registrar)
            aiPlugin = plugin
        }

        if let mdRegistrar = engineBridge.pluginRegistry.registrar(forPlugin: "ModelDownloadPlugin") {
            let mdPlugin = ModelDownloadPlugin()
            mdPlugin.register(with: mdRegistrar)
        }
    }

    override func applicationDidReceiveMemoryWarning(_ application: UIApplication) {
        super.applicationDidReceiveMemoryWarning(application)
        aiPlugin?.releaseModel()
    }
}
