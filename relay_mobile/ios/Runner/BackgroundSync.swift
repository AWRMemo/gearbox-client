import Foundation
import BackgroundTasks

class BackgroundSyncManager {
    static let taskId = "com.gearbox.relay.sync"

    static func register() {
        BGTaskScheduler.shared.register(forTaskWithIdentifier: taskId, using: nil) { task in
            handleTask(task as! BGAppRefreshTask)
        }
    }

    static func schedule() {
        let request = BGAppRefreshTaskRequest(identifier: taskId)
        request.earliestBeginDate = Date(timeIntervalSinceNow: 30 * 60)
        try? BGTaskScheduler.shared.submit(request)
    }

    private static func handleTask(_ task: BGAppRefreshTask) {
        schedule()

        task.expirationHandler = {
            task.setTaskCompleted(success: false)
        }

        // Call FRB sync via notification to Flutter engine
        NotificationCenter.default.post(name: .init("relay://background-sync"), object: nil)

        // Give Flutter 25 seconds to respond, then complete
        DispatchQueue.main.asyncAfter(deadline: .now() + 25) {
            task.setTaskCompleted(success: true)
        }
    }
}
