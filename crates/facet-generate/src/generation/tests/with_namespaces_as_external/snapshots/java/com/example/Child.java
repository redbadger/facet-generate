package com.example;


public final class Child {
    public final com.example2.other.OtherParent external;

    public Child(com.example2.other.OtherParent external) {
        java.util.Objects.requireNonNull(external, "external must not be null");
        this.external = external;
    }

    public boolean equals(Object obj) {
        if (this == obj) return true;
        if (obj == null) return false;
        if (getClass() != obj.getClass()) return false;
        Child other = (Child) obj;
        if (!java.util.Objects.equals(this.external, other.external)) { return false; }
        return true;
    }

    public int hashCode() {
        int value = 7;
        value = 31 * value + (this.external != null ? this.external.hashCode() : 0);
        return value;
    }

    public static final class Builder {
        public com.example2.other.OtherParent external;

        public Child build() {
            return new Child(
                external
            );
        }
    }
}
