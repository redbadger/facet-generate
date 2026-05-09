using CommunityToolkit.Mvvm.ComponentModel;
using Facet.Runtime.Serde;
using System.Collections.Generic;
using System.Collections.ObjectModel;

namespace Example;

public partial class Foo : ObservableObject {
    [ObservableProperty]
    private Guid _id;
    [ObservableProperty]
    private Guid? _maybeId;
}
