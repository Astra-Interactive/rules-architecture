plugins {
    id("org.jetbrains.kotlin.jvm")
    id("ru.astrainteractive.klibs.rules.gradle.linter")
}

kotlin {
    explicitApi()
}

dependencies {
    implementation(project(":structure"))
}
