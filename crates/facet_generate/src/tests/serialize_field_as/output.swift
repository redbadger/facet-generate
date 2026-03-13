import Foundation

struct EditItemViewModelSaveRequest: Codable {
    var context: String
    var values: [EditItemSaveValue]
    var fill_action: AutoFillItemActionRequest?

    init(context: String, values: [EditItemSaveValue], fill_action: AutoFillItemActionRequest?) {
        self.context = context
        self.values = values
        self.fill_action = fill_action
    }
}
