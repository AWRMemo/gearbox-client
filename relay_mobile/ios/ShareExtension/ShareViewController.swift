import UIKit
import Social
import MobileCoreServices

class ShareViewController: UIViewController {
    override func viewDidLoad() {
        super.viewDidLoad()
        handleSharedItem()
    }

    private func handleSharedItem() {
        guard let extensionItems = extensionContext?.inputItems as? [NSExtensionItem],
              let item = extensionItems.first,
              let attachments = item.attachments else {
            completeRequest()
            return
        }

        for provider in attachments {
            if provider.hasItemConformingToTypeIdentifier(kUTTypeText as String) {
                provider.loadItem(forTypeIdentifier: kUTTypeText as String, options: nil) { [weak self] (text, error) in
                    guard let self = self, error == nil else {
                        self?.completeRequest()
                        return
                    }

                    var content = ""
                    if let url = text as? URL {
                        content = "\(url.absoluteString)"
                    } else if let str = text as? String {
                        content = str
                    }

                    if !content.isEmpty {
                        self.saveToAppGroup(content)
                    }
                    self.completeRequest()
                }
                return
            }
        }
        completeRequest()
    }

    private func saveToAppGroup(_ content: String) {
        guard let containerUrl = FileManager.default
            .containerURL(forSecurityApplicationGroupIdentifier: "group.com.gearbox.relay") else { return }

        let sharedFile = containerUrl.appendingPathComponent("shared_content.txt")
        do {
            try content.write(to: sharedFile, atomically: true, encoding: .utf8)
        } catch {
            print("ShareExtension: failed to write shared content: \(error)")
        }
    }

    private func completeRequest() {
        extensionContext?.completeRequest(returningItems: nil, completionHandler: nil)
    }
}