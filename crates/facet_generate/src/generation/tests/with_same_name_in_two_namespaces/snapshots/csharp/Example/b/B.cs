using CommunityToolkit.Mvvm.ComponentModel;
using Facet.Runtime.Serde;
using System.Collections.Generic;
using System.Collections.ObjectModel;

namespace Example.B;

public partial class Child : ObservableObject {
    [ObservableProperty]
    private byte _y;
}

public partial class Parent : ObservableObject {
    [ObservableProperty]
    private Example.A.Child _first;
    [ObservableProperty]
    private Child _second;
}
