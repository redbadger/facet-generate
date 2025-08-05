import Foundation

struct A: Codable {
    var field: UInt32

    init(field: UInt32) {
        self.field = field
    }
}

struct B: Codable {
    var dependsOn: A

    init(dependsOn: A) {
        self.dependsOn = dependsOn
    }
}

struct C: Codable {
    var dependsOn: B

    init(dependsOn: B) {
        self.dependsOn = dependsOn
    }
}

struct D: Codable {
    var dependsOn: C
    var alsoDependsOn: E?

    init(dependsOn: C, alsoDependsOn: E?) {
        self.dependsOn = dependsOn
        self.alsoDependsOn = alsoDependsOn
    }
}

struct E: Codable {
    var dependsOn: D

    init(dependsOn: D) {
        self.dependsOn = dependsOn
    }
}
