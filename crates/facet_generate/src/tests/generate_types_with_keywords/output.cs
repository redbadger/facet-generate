using CommunityToolkit.Mvvm.ComponentModel;
using Facet.Runtime.Serde;
using System.Collections.Generic;
using System.Collections.ObjectModel;

namespace Example;

public abstract record KeywordEnum {
    public sealed record Default() : KeywordEnum;

    public sealed record Case() : KeywordEnum;

    public sealed record Switch(string Value) : KeywordEnum;

    public sealed record Where(int In, string Default) : KeywordEnum {
        public new string Default { get; init; } = Default;
    }

}

/// A struct whose every field is a keyword in at least one target language.
/// Each language escapes only its own reserved words: `import` is escaped in
/// Swift and TypeScript but is a soft keyword in Kotlin, and `type` is
/// contextual everywhere, so both come through bare where they are legal.
public partial class KeywordFields : ObservableObject {
    [ObservableProperty]
    private string _default;
    [ObservableProperty]
    private int _in;
    [ObservableProperty]
    private bool _class;
    [ObservableProperty]
    private string _object;
    [ObservableProperty]
    private bool _static;
    [ObservableProperty]
    private string _let;
    [ObservableProperty]
    private int _when;
    [ObservableProperty]
    private bool _is;
    [ObservableProperty]
    private string _fun;
    [ObservableProperty]
    private string _operator;
    [ObservableProperty]
    private string _import;
    [ObservableProperty]
    private string _type;
    [ObservableProperty]
    private string? _function;
    /// A tuple field: the Swift plugin derives `whereField0` / `whereField1`
    /// locals from this name, which must stay unescaped.
    [ObservableProperty]
    private (int, string) _where;
}

/// Newtype struct — its member is named `value`, a Kotlin soft keyword that
/// must not be escaped.
public partial class KeywordNewType : ObservableObject {
    [ObservableProperty]
    private string _value;
}

/// Tuple struct — its members are named `field0`, `field1`, which are never
/// keywords.
public partial class KeywordTuple : ObservableObject {
    [ObservableProperty]
    private string _field0;
    [ObservableProperty]
    private int _field1;
}
