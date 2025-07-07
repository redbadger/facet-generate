// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::collections::HashSet;

use crate::reflection::format::{ContainerFormat, Format, FormatHolder, Named, VariantFormat};

#[test]
fn test_format_visiting() {
    use Format::*;

    let format = ContainerFormat::Enum(
        vec![(
            0,
            Named {
                name: "foo".into(),
                value: VariantFormat::Tuple(vec![
                    TypeName("foo".into()),
                    TypeName("bar".into()),
                    Seq(Box::new(TypeName("foo".into()))),
                ]),
            },
        )]
        .into_iter()
        .collect(),
    );
    let mut names = HashSet::new();
    format
        .visit(&mut |f| {
            if let TypeName(x) = f {
                // Insert a &str borrowed from `format`.
                names.insert(x.to_legacy_string());
            }
            Ok(())
        })
        .unwrap();
    assert_eq!(names.len(), 2);

    assert!(VariantFormat::unknown().visit(&mut |_| Ok(())).is_err());
    assert!(Format::unknown().visit(&mut |_| Ok(())).is_err());
}
