pluginManagement {
    repositories {
        mavenLocal()
        mavenCentral()
        gradlePluginPortal()
    }
    plugins {
        id("org.jetbrains.kotlin.jvm") version System.getProperty("klibsKotlinVersion")
        id("ru.astrainteractive.klibs.rules.gradle.linter") version System.getProperty("klibsRulesVersion")
    }
}

dependencyResolutionManagement {
    repositories {
        mavenLocal()
        mavenCentral()
    }
}

rootProject.name = "rules-architecture-self-check"

include(":detekt")
include(":gradle-plugin")
include(":konsist")
include(":structure")

listOf("detekt", "gradle-plugin", "konsist", "structure").forEach { module ->
    project(":$module").projectDir = file("build/units/$module")
}
