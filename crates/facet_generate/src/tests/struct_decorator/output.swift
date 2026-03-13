import Foundation

struct BestHockeyTeams: Codable {
    var PittsburghPenguins: UInt32
    var Lies: String

    init(PittsburghPenguins: UInt32, Lies: String) {
        self.PittsburghPenguins = PittsburghPenguins
        self.Lies = Lies
    }
}

struct BestHockeyTeams1: Codable, Equatable {
    var PittsburghPenguins: UInt32
    var Lies: String

    init(PittsburghPenguins: UInt32, Lies: String) {
        self.PittsburghPenguins = PittsburghPenguins
        self.Lies = Lies
    }
}

struct BestHockeyTeams2: Codable, Comparable, Equatable, Hashable {
    var PittsburghPenguins: UInt32
    var Lies: String

    init(PittsburghPenguins: UInt32, Lies: String) {
        self.PittsburghPenguins = PittsburghPenguins
        self.Lies = Lies
    }
}

struct BestHockeyTeams3: Codable {
    var PittsburghPenguins: UInt32
    var Lies: String

    init(PittsburghPenguins: UInt32, Lies: String) {
        self.PittsburghPenguins = PittsburghPenguins
        self.Lies = Lies
    }
}

struct BestHockeyTeams4: Codable, Equatable, Hashable {
    var PittsburghPenguins: UInt32
    var Lies: String

    init(PittsburghPenguins: UInt32, Lies: String) {
        self.PittsburghPenguins = PittsburghPenguins
        self.Lies = Lies
    }
}
