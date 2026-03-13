import Foundation

enum BestHockeyTeams: String, Codable {
    case pittsburghPenguins = "PittsburghPenguins"
}

enum BestHockeyTeams1: String, Codable, Equatable {
    case pittsburghPenguins = "PittsburghPenguins"
}

enum BestHockeyTeams2: String, Codable, Comparable, Equatable, Hashable {
    case pittsburghPenguins = "PittsburghPenguins"
}

enum BestHockeyTeams3: String, Codable {
    case pittsburghPenguins = "PittsburghPenguins"
}

enum BestHockeyTeams4: String, Codable, Equatable, Hashable {
    case pittsburghPenguins = "PittsburghPenguins"
}
