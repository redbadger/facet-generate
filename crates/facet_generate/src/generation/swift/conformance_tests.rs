//! Unit tests for [`Conformance`]: which types of a registry conform to
//! `Hashable` and `Equatable`, across namespaces, cycles and external
//! packages.

use std::collections::BTreeSet;

use facet::Facet;

use super::*;
use crate::{
    self as fg,
    generation::{ExternalPackage, PackageLocation},
    reflect,
};

fn names(set: &BTreeSet<QualifiedTypeName>) -> Vec<String> {
    set.iter().map(ToString::to_string).collect()
}

/// Non-conformance propagates through two namespaces: `Top` holds a
/// `kit::Middle`, which holds an `other::Leaf` with a native tuple (not
/// `Hashable`) and an `other::Void` with a `()` (neither).
#[test]
fn non_conformance_propagates_across_namespaces() {
    #[derive(Facet)]
    #[facet(fg::namespace = "other")]
    struct Leaf {
        pair: (u32, u32),
    }

    #[derive(Facet)]
    #[facet(fg::namespace = "other")]
    struct Void {
        unit: (),
    }

    #[derive(Facet)]
    #[facet(fg::namespace = "kit")]
    struct Middle {
        leaves: Vec<Leaf>,
    }

    #[derive(Facet)]
    #[facet(fg::namespace = "kit")]
    struct Voids {
        voids: Option<Box<Void>>,
    }

    #[derive(Facet)]
    struct Top {
        middle: Option<Middle>,
    }

    #[derive(Facet)]
    struct TopVoids {
        voids: std::collections::BTreeMap<String, Voids>,
    }

    let conformance = Conformance::of(&reflect!(Top, TopVoids).unwrap(), &ExternalPackages::new());

    insta::assert_debug_snapshot!(names(&conformance.decided), @r#"
    [
        "ROOT::Top",
        "ROOT::TopVoids",
        "kit::Middle",
        "kit::Voids",
        "other::Leaf",
        "other::Void",
    ]
    "#);
    insta::assert_debug_snapshot!(names(&conformance.hashable), @"[]");
    insta::assert_debug_snapshot!(names(&conformance.equatable), @r#"
    [
        "ROOT::Top",
        "kit::Middle",
        "other::Leaf",
    ]
    "#);
}

/// A cycle conforms only when nothing else its types hold stops it, whichever
/// type of it is looked at first: `Ping` comes first and holds a native tuple
/// as well as a `Pong`, which holds only a `Ping`, so neither is `Hashable`.
/// `Tick` and `Tock` hold only each other, so both are.
#[test]
fn a_cycle_conforms_only_when_every_type_in_it_does() {
    #[derive(Facet)]
    #[repr(C)]
    #[allow(dead_code)]
    enum Ping {
        Pong(Box<Pong>),
        Pair((u32, u32)),
    }

    #[derive(Facet)]
    #[repr(C)]
    #[allow(dead_code)]
    enum Pong {
        Done,
        Ping(Box<Ping>),
    }

    #[derive(Facet)]
    #[repr(C)]
    #[allow(dead_code)]
    enum Tick {
        Done,
        Tock(Box<Tock>),
    }

    #[derive(Facet)]
    #[repr(C)]
    #[allow(dead_code)]
    enum Tock {
        Done,
        Tick(Box<Tick>),
    }

    #[derive(Facet)]
    struct HoldsPong {
        pong: Pong,
        tick: Tick,
    }

    let conformance = Conformance::of(&reflect!(HoldsPong).unwrap(), &ExternalPackages::new());

    insta::assert_debug_snapshot!(names(&conformance.hashable), @r#"
    [
        "ROOT::Tick",
        "ROOT::Tock",
    ]
    "#);
    insta::assert_debug_snapshot!(names(&conformance.equatable), @r#"
    [
        "ROOT::HoldsPong",
        "ROOT::Ping",
        "ROOT::Pong",
        "ROOT::Tick",
        "ROOT::Tock",
    ]
    "#);
}

/// A type is `Equatable` only when the emitter can declare it so: a native
/// tuple field gets a hand-written `==`, but an array of native tuples can't
/// be compared with `==`, so a type holding one, and every type holding that,
/// is not `Equatable`.
#[test]
fn equatable_follows_the_rules_a_type_is_declared_with() {
    #[derive(Facet)]
    struct Pair {
        pair: (u32, u32),
    }

    #[derive(Facet)]
    struct Pairs {
        pairs: Vec<(u32, u32)>,
    }

    #[derive(Facet)]
    struct Holder {
        pair: Pair,
        pairs: Pairs,
    }

    let conformance = Conformance::of(&reflect!(Holder).unwrap(), &ExternalPackages::new());

    insta::assert_debug_snapshot!(names(&conformance.hashable), @"[]");
    insta::assert_debug_snapshot!(names(&conformance.equatable), @r#"
    [
        "ROOT::Pair",
    ]
    "#);
}

/// The types of an external package are generated elsewhere, so their
/// conformance is not decided here: a type holding one assumes it conforms.
#[test]
fn a_type_from_an_external_package_is_assumed_to_conform() {
    #[derive(Facet)]
    #[facet(fg::namespace = "api")]
    struct Holder {
        t: (u32, u32),
    }

    #[derive(Facet)]
    struct SwHash {
        h: Holder,
    }

    let external_packages = [ExternalPackage {
        for_namespace: "api".to_string(),
        location: PackageLocation::Path("../Api".to_string()),
        module_name: None,
        version: None,
    }]
    .into_iter()
    .map(|p| (p.for_namespace.clone(), p))
    .collect();

    let conformance = Conformance::of(&reflect!(SwHash).unwrap(), &external_packages);

    insta::assert_debug_snapshot!(names(&conformance.decided), @r#"
    [
        "ROOT::SwHash",
    ]
    "#);
    insta::assert_debug_snapshot!(names(&conformance.hashable), @r#"
    [
        "ROOT::SwHash",
    ]
    "#);
}
