#![allow(clippy::too_many_lines)]
use facet::Facet;

use crate as fg;
use crate::reflect;

use super::*;

#[test]
fn single_namespace() {
    #[derive(Facet)]
    struct ChildOne {
        child: GrandChild,
    }

    #[derive(Facet)]
    struct ChildTwo {
        field: String,
    }

    #[derive(Facet)]
    struct GrandChild {
        field: String,
    }

    #[derive(Facet)]
    struct Parent {
        one: ChildOne,
        two: ChildTwo,
    }

    let registries = split("Root", &reflect!(Parent).unwrap());
    insta::assert_debug_snapshot!(registries, @r#"
    {
        Module(
            CodeGeneratorConfig {
                module_name: "Root",
                external_definitions: {},
                external_packages: {},
                comments: {},
                package_manifest: true,
                features: {},
                indent: Space(
                    4,
                ),
                used_format_types: {},
                referenced_namespaces: {},
                unit_variant_enums: {},
            },
        ): {
            QualifiedTypeName {
                namespace: Root,
                name: "ChildOne",
            }: Struct(
                [
                    Named {
                        name: "child",
                        doc: Doc(
                            [],
                        ),
                        value: TypeName(
                            QualifiedTypeName {
                                namespace: Root,
                                name: "GrandChild",
                            },
                        ),
                    },
                ],
                Doc(
                    [],
                ),
            ),
            QualifiedTypeName {
                namespace: Root,
                name: "ChildTwo",
            }: Struct(
                [
                    Named {
                        name: "field",
                        doc: Doc(
                            [],
                        ),
                        value: Str,
                    },
                ],
                Doc(
                    [],
                ),
            ),
            QualifiedTypeName {
                namespace: Root,
                name: "GrandChild",
            }: Struct(
                [
                    Named {
                        name: "field",
                        doc: Doc(
                            [],
                        ),
                        value: Str,
                    },
                ],
                Doc(
                    [],
                ),
            ),
            QualifiedTypeName {
                namespace: Root,
                name: "Parent",
            }: Struct(
                [
                    Named {
                        name: "one",
                        doc: Doc(
                            [],
                        ),
                        value: TypeName(
                            QualifiedTypeName {
                                namespace: Root,
                                name: "ChildOne",
                            },
                        ),
                    },
                    Named {
                        name: "two",
                        doc: Doc(
                            [],
                        ),
                        value: TypeName(
                            QualifiedTypeName {
                                namespace: Root,
                                name: "ChildTwo",
                            },
                        ),
                    },
                ],
                Doc(
                    [],
                ),
            ),
        },
    }
    "#);
}

#[test]
#[allow(clippy::too_many_lines)]
fn root_namespace_with_two_child_namespaces() {
    #[derive(Facet)]
    #[facet(fg::namespace = "one")]
    struct ChildOne {
        child: GrandChild,
    }

    #[derive(Facet)]
    #[facet(fg::namespace = "two")]
    struct ChildTwo {
        field: String,
    }

    #[derive(Facet)]
    #[facet(fg::namespace = "one")]
    struct GrandChild {
        field: String,
    }

    #[derive(Facet)]
    struct Parent {
        one: ChildOne,
        two: ChildTwo,
    }

    let registries = split("Root", &reflect!(Parent).unwrap());
    insta::assert_debug_snapshot!(registries, @r#"
    {
        Module(
            CodeGeneratorConfig {
                module_name: "Root",
                external_definitions: {
                    "one": [
                        "ChildOne",
                    ],
                    "two": [
                        "ChildTwo",
                    ],
                },
                external_packages: {},
                comments: {},
                package_manifest: true,
                features: {},
                indent: Space(
                    4,
                ),
                used_format_types: {},
                referenced_namespaces: {},
                unit_variant_enums: {},
            },
        ): {
            QualifiedTypeName {
                namespace: Root,
                name: "Parent",
            }: Struct(
                [
                    Named {
                        name: "one",
                        doc: Doc(
                            [],
                        ),
                        value: TypeName(
                            QualifiedTypeName {
                                namespace: Named(
                                    "one",
                                ),
                                name: "ChildOne",
                            },
                        ),
                    },
                    Named {
                        name: "two",
                        doc: Doc(
                            [],
                        ),
                        value: TypeName(
                            QualifiedTypeName {
                                namespace: Named(
                                    "two",
                                ),
                                name: "ChildTwo",
                            },
                        ),
                    },
                ],
                Doc(
                    [],
                ),
            ),
        },
        Module(
            CodeGeneratorConfig {
                module_name: "one",
                external_definitions: {},
                external_packages: {},
                comments: {},
                package_manifest: true,
                features: {},
                indent: Space(
                    4,
                ),
                used_format_types: {},
                referenced_namespaces: {},
                unit_variant_enums: {},
            },
        ): {
            QualifiedTypeName {
                namespace: Named(
                    "one",
                ),
                name: "ChildOne",
            }: Struct(
                [
                    Named {
                        name: "child",
                        doc: Doc(
                            [],
                        ),
                        value: TypeName(
                            QualifiedTypeName {
                                namespace: Named(
                                    "one",
                                ),
                                name: "GrandChild",
                            },
                        ),
                    },
                ],
                Doc(
                    [],
                ),
            ),
            QualifiedTypeName {
                namespace: Named(
                    "one",
                ),
                name: "GrandChild",
            }: Struct(
                [
                    Named {
                        name: "field",
                        doc: Doc(
                            [],
                        ),
                        value: Str,
                    },
                ],
                Doc(
                    [],
                ),
            ),
        },
        Module(
            CodeGeneratorConfig {
                module_name: "two",
                external_definitions: {},
                external_packages: {},
                comments: {},
                package_manifest: true,
                features: {},
                indent: Space(
                    4,
                ),
                used_format_types: {},
                referenced_namespaces: {},
                unit_variant_enums: {},
            },
        ): {
            QualifiedTypeName {
                namespace: Named(
                    "two",
                ),
                name: "ChildTwo",
            }: Struct(
                [
                    Named {
                        name: "field",
                        doc: Doc(
                            [],
                        ),
                        value: Str,
                    },
                ],
                Doc(
                    [],
                ),
            ),
        },
    }
    "#);
}

#[test]
fn same_namespace_with_external_dependency_bug_regression() {
    // This test reproduces the specific bug where external dependencies
    // were lost when multiple types in the same namespace were processed
    // and the first type didn't have external dependencies.

    #[derive(Facet)]
    #[facet(fg::namespace = "api")]
    struct GrandChild {
        test: String,
    }

    #[derive(Facet)]
    struct Child {
        api: GrandChild,
    }

    #[derive(Facet)]
    struct Parent {
        event: Child,
    }

    let registries = split("App", &reflect!(Parent).unwrap());

    // The App module should contain external dependencies to "api" namespace
    // even though Parent itself doesn't directly reference it - Child does
    let app_module = registries
        .keys()
        .find(|m| m.config().module_name() == "App")
        .unwrap();
    let external_deps = &app_module.config().external_definitions;

    // This should NOT be empty - it should include the "api" dependency
    assert!(
        !external_deps.is_empty(),
        "App module should have external dependencies"
    );
    assert!(
        external_deps.contains_key("api"),
        "App module should depend on api namespace"
    );
    assert_eq!(
        external_deps["api"],
        vec!["GrandChild"],
        "App module should reference GrandChild from api"
    );

    insta::assert_debug_snapshot!(registries, @r#"
    {
        Module(
            CodeGeneratorConfig {
                module_name: "App",
                external_definitions: {
                    "api": [
                        "GrandChild",
                    ],
                },
                external_packages: {},
                comments: {},
                package_manifest: true,
                features: {},
                indent: Space(
                    4,
                ),
                used_format_types: {},
                referenced_namespaces: {},
                unit_variant_enums: {},
            },
        ): {
            QualifiedTypeName {
                namespace: Root,
                name: "Child",
            }: Struct(
                [
                    Named {
                        name: "api",
                        doc: Doc(
                            [],
                        ),
                        value: TypeName(
                            QualifiedTypeName {
                                namespace: Named(
                                    "api",
                                ),
                                name: "GrandChild",
                            },
                        ),
                    },
                ],
                Doc(
                    [],
                ),
            ),
            QualifiedTypeName {
                namespace: Root,
                name: "Parent",
            }: Struct(
                [
                    Named {
                        name: "event",
                        doc: Doc(
                            [],
                        ),
                        value: TypeName(
                            QualifiedTypeName {
                                namespace: Root,
                                name: "Child",
                            },
                        ),
                    },
                ],
                Doc(
                    [],
                ),
            ),
        },
        Module(
            CodeGeneratorConfig {
                module_name: "api",
                external_definitions: {},
                external_packages: {},
                comments: {},
                package_manifest: true,
                features: {},
                indent: Space(
                    4,
                ),
                used_format_types: {},
                referenced_namespaces: {},
                unit_variant_enums: {},
            },
        ): {
            QualifiedTypeName {
                namespace: Named(
                    "api",
                ),
                name: "GrandChild",
            }: Struct(
                [
                    Named {
                        name: "test",
                        doc: Doc(
                            [],
                        ),
                        value: Str,
                    },
                ],
                Doc(
                    [],
                ),
            ),
        },
    }
    "#);
}
