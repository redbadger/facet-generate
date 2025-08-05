import Foundation
import UIKit

struct SomeStruct: Codable {
    var field: String

    init(field: String) {
        self.field = field
    }
}
