using CommunityToolkit.Mvvm.ComponentModel;
using Facet.Runtime.Serde;
using System.Collections.Generic;
using System.Collections.ObjectModel;

namespace Example;

public partial class Root : ObservableObject {
    [ObservableProperty]
    private Example.B.Parent _parent;
}
