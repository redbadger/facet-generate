package com.example;


public final class Child {
    public final String name;

    public Child(String name) {
        java.util.Objects.requireNonNull(name, "name must not be null");
        this.name = name;
    }

    public boolean equals(Object obj) {
        if (this == obj) return true;
        if (obj == null) return false;
        if (getClass() != obj.getClass()) return false;
        Child other = (Child) obj;
        if (!java.util.Objects.equals(this.name, other.name)) { return false; }
        return true;
    }

    public int hashCode() {
        int value = 7;
        value = 31 * value + (this.name != null ? this.name.hashCode() : 0);
        return value;
    }

    public static final class Builder {
        public String name;

        public Child build() {
            return new Child(
                name
            );
        }
    }
}
