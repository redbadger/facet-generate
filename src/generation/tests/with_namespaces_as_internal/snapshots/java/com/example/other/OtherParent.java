package com.example.other;


public abstract class OtherParent {

    public static final class Child extends OtherParent {
        public final OtherChild value;

        public Child(OtherChild value) {
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
            public OtherChild value;

            public Child build() {
                return new Child(
                    value
                );
            }
        }
    }
}

