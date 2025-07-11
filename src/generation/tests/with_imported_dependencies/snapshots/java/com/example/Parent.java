package com.example;


public abstract class Parent {

    public static final class Child extends Parent {
        public final com.example.Child value;

        public Child(com.example.Child value) {
            java.util.Objects.requireNonNull(value, "value must not be null");
            this.value = value;
        }

        public boolean equals(Object obj) {
            if (this == obj) return true;
            if (obj == null) return false;
            if (getClass() != obj.getClass()) return false;
            Child other = (Child) obj;
            if (!java.util.Objects.equals(this.value, other.value)) { return false; }
            return true;
        }

        public int hashCode() {
            int value = 7;
            value = 31 * value + (this.value != null ? this.value.hashCode() : 0);
            return value;
        }

        public static final class Builder {
            public com.example.Child value;

            public Child build() {
                return new Child(
                    value
                );
            }
        }
    }
}

