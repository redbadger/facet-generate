package example.com;


public final class UnitStruct {
    public UnitStruct() {
    }

    public boolean equals(Object obj) {
        if (this == obj) return true;
        if (obj == null) return false;
        if (getClass() != obj.getClass()) return false;
        UnitStruct other = (UnitStruct) obj;
        return true;
    }

    public int hashCode() {
        int value = 7;
        return value;
    }

    public static final class Builder {
        public UnitStruct build() {
            return new UnitStruct(
            );
        }
    }
}
