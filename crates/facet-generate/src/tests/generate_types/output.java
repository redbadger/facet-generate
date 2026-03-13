package com.example;


public final class CustomType {
    public CustomType() {
    }

    public boolean equals(Object obj) {
        if (this == obj) return true;
        if (obj == null) return false;
        if (getClass() != obj.getClass()) return false;
        CustomType other = (CustomType) obj;
        return true;
    }

    public int hashCode() {
        int value = 7;
        return value;
    }

    public static final class Builder {
        public CustomType build() {
            return new CustomType(
            );
        }
    }
}

public final class Types {
    public final String s;
    public final String static_s;
    public final Byte int8;
    public final Float float;
    public final Double double;
    public final java.util.List<String> array;
    public final java.util.@com.novi.serde.ArrayLen(length=4) List<String> fixed_length_array;
    public final java.util.Map<String, Integer> dictionary;
    public final java.util.Optional<java.util.Map<String, Integer>> optional_dictionary;
    public final CustomType custom_type;

    public Types(String s, String static_s, Byte int8, Float float, Double double, java.util.List<String> array, java.util.@com.novi.serde.ArrayLen(length=4) List<String> fixed_length_array, java.util.Map<String, Integer> dictionary, java.util.Optional<java.util.Map<String, Integer>> optional_dictionary, CustomType custom_type) {
        java.util.Objects.requireNonNull(s, "s must not be null");
        java.util.Objects.requireNonNull(static_s, "static_s must not be null");
        java.util.Objects.requireNonNull(int8, "int8 must not be null");
        java.util.Objects.requireNonNull(float, "float must not be null");
        java.util.Objects.requireNonNull(double, "double must not be null");
        java.util.Objects.requireNonNull(array, "array must not be null");
        java.util.Objects.requireNonNull(fixed_length_array, "fixed_length_array must not be null");
        java.util.Objects.requireNonNull(dictionary, "dictionary must not be null");
        java.util.Objects.requireNonNull(optional_dictionary, "optional_dictionary must not be null");
        java.util.Objects.requireNonNull(custom_type, "custom_type must not be null");
        this.s = s;
        this.static_s = static_s;
        this.int8 = int8;
        this.float = float;
        this.double = double;
        this.array = array;
        this.fixed_length_array = fixed_length_array;
        this.dictionary = dictionary;
        this.optional_dictionary = optional_dictionary;
        this.custom_type = custom_type;
    }

    public boolean equals(Object obj) {
        if (this == obj) return true;
        if (obj == null) return false;
        if (getClass() != obj.getClass()) return false;
        Types other = (Types) obj;
        if (!java.util.Objects.equals(this.s, other.s)) { return false; }
        if (!java.util.Objects.equals(this.static_s, other.static_s)) { return false; }
        if (!java.util.Objects.equals(this.int8, other.int8)) { return false; }
        if (!java.util.Objects.equals(this.float, other.float)) { return false; }
        if (!java.util.Objects.equals(this.double, other.double)) { return false; }
        if (!java.util.Objects.equals(this.array, other.array)) { return false; }
        if (!java.util.Objects.equals(this.fixed_length_array, other.fixed_length_array)) { return false; }
        if (!java.util.Objects.equals(this.dictionary, other.dictionary)) { return false; }
        if (!java.util.Objects.equals(this.optional_dictionary, other.optional_dictionary)) { return false; }
        if (!java.util.Objects.equals(this.custom_type, other.custom_type)) { return false; }
        return true;
    }

    public int hashCode() {
        int value = 7;
        value = 31 * value + (this.s != null ? this.s.hashCode() : 0);
        value = 31 * value + (this.static_s != null ? this.static_s.hashCode() : 0);
        value = 31 * value + (this.int8 != null ? this.int8.hashCode() : 0);
        value = 31 * value + (this.float != null ? this.float.hashCode() : 0);
        value = 31 * value + (this.double != null ? this.double.hashCode() : 0);
        value = 31 * value + (this.array != null ? this.array.hashCode() : 0);
        value = 31 * value + (this.fixed_length_array != null ? this.fixed_length_array.hashCode() : 0);
        value = 31 * value + (this.dictionary != null ? this.dictionary.hashCode() : 0);
        value = 31 * value + (this.optional_dictionary != null ? this.optional_dictionary.hashCode() : 0);
        value = 31 * value + (this.custom_type != null ? this.custom_type.hashCode() : 0);
        return value;
    }

    public static final class Builder {
        public String s;
        public String static_s;
        public Byte int8;
        public Float float;
        public Double double;
        public java.util.List<String> array;
        public java.util.@com.novi.serde.ArrayLen(length=4) List<String> fixed_length_array;
        public java.util.Map<String, Integer> dictionary;
        public java.util.Optional<java.util.Map<String, Integer>> optional_dictionary;
        public CustomType custom_type;

        public Types build() {
            return new Types(
                s,
                static_s,
                int8,
                float,
                double,
                array,
                fixed_length_array,
                dictionary,
                optional_dictionary,
                custom_type
            );
        }
    }
}
