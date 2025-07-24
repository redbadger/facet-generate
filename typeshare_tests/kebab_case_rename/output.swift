import Foundation

/// This is a comment.
struct Things: Codable {
    var bla: String
    var label: String?
    var label_left: String?

    enum CodingKeys: String, CodingKey {
        case bla
        case label
        case label_left = "label-left"
    }

    init(bla: String, label: String?, label_left: String?) {
        self.bla = bla
        self.label = label
        self.label_left = label_left
    }
}
