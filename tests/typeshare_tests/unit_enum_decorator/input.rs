#![expect(unused)]

use facet::Facet;

#[derive(Facet)]
#[repr(C)]
pub enum BestHockeyTeams {
    PittsburghPenguins,
}

#[derive(Facet)]
#[facet(swift = "Equatable")]
#[repr(C)]
pub enum BestHockeyTeams1 {
    PittsburghPenguins,
}

#[derive(Facet)]
#[facet(swift = "Equatable, Comparable, Hashable")]
#[repr(C)]
pub enum BestHockeyTeams2 {
    PittsburghPenguins,
}

#[derive(Facet)]
#[facet(kotlin = "idk")]
#[repr(C)]
pub enum BestHockeyTeams3 {
    PittsburghPenguins,
}

#[derive(Facet)]
#[facet(swift = "Equatable", swift = "Hashable")]
#[repr(C)]
pub enum BestHockeyTeams4 {
    PittsburghPenguins,
}
