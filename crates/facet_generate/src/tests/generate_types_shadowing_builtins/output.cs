using CommunityToolkit.Mvvm.ComponentModel;
using Facet.Runtime.Serde;
using System.Collections.Generic;
using System.Collections.ObjectModel;

namespace Example;

public abstract record BoolResult {
    public sealed record Ok(bool Value) : BoolResult;

    public sealed record Err(string Value) : BoolResult;

}

public partial class Delete : ObservableObject {
    [ObservableProperty]
    private string _key;
}

public partial class Exists : ObservableObject {
    [ObservableProperty]
    private string _key;
}

public partial class Get : ObservableObject {
    [ObservableProperty]
    private string _key;
}

public partial class Keys : ObservableObject {
    [ObservableProperty]
    private ObservableCollection<string> _items;
    [ObservableProperty]
    private ulong _nextCursor;
}

public abstract record KeysResult {
    public sealed record Ok(Keys Value) : KeysResult;

    public sealed record Err(string Value) : KeysResult;

}

public partial class ListKeys : ObservableObject {
    [ObservableProperty]
    private string _prefix;
    [ObservableProperty]
    private ulong _cursor;
}

public partial class Set : ObservableObject {
    [ObservableProperty]
    private string _key;
    [ObservableProperty]
    private byte[] _value;
}

public partial class Store : ObservableObject {
    [ObservableProperty]
    private HashSet<string> _tags;
    [ObservableProperty]
    private Dictionary<string, string> _entries;
    [ObservableProperty]
    private ObservableCollection<byte> _blob;
    [ObservableProperty]
    private (int, string) _pair;
}

public abstract record ValueResult {
    public sealed record Ok(ObservableCollection<byte>? Value) : ValueResult;

    public sealed record Err(string Value) : ValueResult;

}
