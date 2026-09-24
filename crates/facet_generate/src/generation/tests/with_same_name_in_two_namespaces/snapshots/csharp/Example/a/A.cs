using CommunityToolkit.Mvvm.ComponentModel;
using Facet.Runtime.Serde;
using System.Collections.Generic;
using System.Collections.ObjectModel;

namespace Example.A;

public partial class Child : ObservableObject {
    [ObservableProperty]
    private byte _x;
}
