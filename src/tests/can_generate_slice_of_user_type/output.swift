import Foundation

struct Video: Codable {
    var tags: [Tag]

    init(tags: [Tag]) {
        self.tags = tags
    }
}
