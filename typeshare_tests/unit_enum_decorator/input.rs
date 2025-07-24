#[derive(Facet)]
pub enum BestHockeyTeams {
    PittsburghPenguins,
}

#[derive(Facet)]
#[facet(swift = "Equatable")]
pub enum BestHockeyTeams1 {
    PittsburghPenguins,
}

#[derive(Facet)]
#[facet(swift = "Equatable, Comparable, Hashable")]
pub enum BestHockeyTeams2 {
    PittsburghPenguins,
}

#[derive(Facet)]
#[facet(kotlin = "idk")]
pub enum BestHockeyTeams3 {
    PittsburghPenguins,
}

#[derive(Facet)]
#[facet(swift = "Equatable", swift = "Hashable")]
pub enum BestHockeyTeams4 {
    PittsburghPenguins,
}
