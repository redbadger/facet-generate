plugins {
    kotlin("jvm") version "2.2.0"
    kotlin("plugin.serialization") version "2.2.0"
    `java-library`
}

group = "com.example"
version = "1.0.0"

repositories {
    mavenCentral()
}

dependencies {
    implementation("org.jetbrains.kotlinx:kotlinx-serialization-json:1.9.0")
}

tasks.withType<Jar> {
    manifest {
        attributes["Implementation-Title"] = "com.example"
        attributes["Implementation-Version"] = "1.0.0"
    }
}
