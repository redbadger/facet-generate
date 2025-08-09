package com.example;


public final class MyStruct {
    public final Integer a;
    public final Integer c;

    public MyStruct(Integer a, Integer c) {
        java.util.Objects.requireNonNull(a, "a must not be null");
        java.util.Objects.requireNonNull(c, "c must not be null");
        this.a = a;
        this.c = c;
    }

    public boolean equals(Object obj) {
        if (this == obj) return true;
        if (obj == null) return false;
        if (getClass() != obj.getClass()) return false;
        MyStruct other = (MyStruct) obj;
        if (!java.util.Objects.equals(this.a, other.a)) { return false; }
        if (!java.util.Objects.equals(this.c, other.c)) { return false; }
        return true;
    }

    public int hashCode() {
        int value = 7;
        value = 31 * value + (this.a != null ? this.a.hashCode() : 0);
        value = 31 * value + (this.c != null ? this.c.hashCode() : 0);
        return value;
    }

    public static final class Builder {
        public Integer a;
        public Integer c;

        public MyStruct build() {
            return new MyStruct(
                a,
                c
            );
        }
    }
}
