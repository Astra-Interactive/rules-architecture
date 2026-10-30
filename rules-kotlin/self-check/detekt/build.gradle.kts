plugins {
    id("org.jetbrains.kotlin.jvm")
    `maven-publish`
    id("ru.astrainteractive.klibs.rules.gradle.linter")
}

kotlin {
    explicitApi()
}

linter {
    entryPointWords.addAll("detekt", "gradle", "konsist")
    technologyWords.add("structure")
}
