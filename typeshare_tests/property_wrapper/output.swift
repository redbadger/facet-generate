import Foundation

struct MyStruct: Codable {
    var field: String
    @Yolo var wrapped_field: String
    var another_field: String

    init(field: String, wrapped_field: String, another_field: String) {
        self.field = field
        self.wrapped_field = wrapped_field
        self.another_field = another_field
    }
}
