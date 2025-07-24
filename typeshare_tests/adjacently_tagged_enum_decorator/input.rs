#[derive(Facet)]
#[facet(tag = "type", content = "content")]
pub enum BestHockeyTeams {
    PittsburghPenguins,
    Lies(String),
}
#[derive(Facet)]
#[facet(swift = "Equatable")]
#[facet(tag = "type", content = "content")]
pub enum BestHockeyTeams1 {
    PittsburghPenguins,
    Lies(String),
}

#[derive(Facet)]
#[facet(swift = "Equatable, Codable, Comparable, Hashable")]
#[facet(tag = "type", content = "content")]
pub enum BestHockeyTeams2 {
    PittsburghPenguins,
    Lies(String),
}

#[derive(Facet)]
#[facet(kotlin = "idk")]
#[facet(tag = "type", content = "content")]
pub enum BestHockeyTeams3 {
    PittsburghPenguins,
    Lies(String),
}

#[derive(Facet)]
#[facet(swift = "Equatable", swift = "Hashable")]
#[facet(tag = "type", content = "content")]
pub enum BestHockeyTeams4 {
    PittsburghPenguins,
    Lies(String),
}
