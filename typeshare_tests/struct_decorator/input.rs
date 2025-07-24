#[derive(Facet)]
pub struct BestHockeyTeams {
    PittsburghPenguins: u32,
    Lies: String,
}
#[derive(Facet)]
#[facet(swift = "Equatable")]
pub struct BestHockeyTeams1 {
    PittsburghPenguins: u32,
    Lies: String,
}

#[derive(Facet)]
#[facet(swift = "Equatable, Codable, Comparable, Hashable")]
pub struct BestHockeyTeams2 {
    PittsburghPenguins: u32,
    Lies: String,
}

#[derive(Facet)]
#[facet(kotlin = "idk")]
pub struct BestHockeyTeams3 {
    PittsburghPenguins: u32,
    Lies: String,
}

#[derive(Facet)]
#[facet(swift = "Equatable", swift = "Hashable")]
pub struct BestHockeyTeams4 {
    PittsburghPenguins: u32,
    Lies: String,
}
