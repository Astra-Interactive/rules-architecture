plugins {
    id("org.jetbrains.kotlin.jvm")
    `maven-publish`
    id("ru.astrainteractive.klibs.rules.gradle.linter")
}

kotlin {
    explicitApi()
}

dependencies {
    implementation(project(":structure"))
}

linter {
    entryPointWords.addAll("detekt", "gradle", "konsist")
    technologyWords.add("structure")
}
