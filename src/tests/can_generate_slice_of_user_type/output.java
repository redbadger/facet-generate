package com.example;


public final class Tag {
    public Tag() {
    }

    public boolean equals(Object obj) {
        if (this == obj) return true;
        if (obj == null) return false;
        if (getClass() != obj.getClass()) return false;
        Tag other = (Tag) obj;
        return true;
    }

    public int hashCode() {
        int value = 7;
        return value;
    }

    public static final class Builder {
        public Tag build() {
            return new Tag(
            );
        }
    }
}

public final class Video {
    public final java.util.List<Tag> tags;

    public Video(java.util.List<Tag> tags) {
        java.util.Objects.requireNonNull(tags, "tags must not be null");
        this.tags = tags;
    }

    public boolean equals(Object obj) {
        if (this == obj) return true;
        if (obj == null) return false;
        if (getClass() != obj.getClass()) return false;
        Video other = (Video) obj;
        if (!java.util.Objects.equals(this.tags, other.tags)) { return false; }
        return true;
    }

    public int hashCode() {
        int value = 7;
        value = 31 * value + (this.tags != null ? this.tags.hashCode() : 0);
        return value;
    }

    public static final class Builder {
        public java.util.List<Tag> tags;

        public Video build() {
            return new Video(
                tags
            );
        }
    }
}
