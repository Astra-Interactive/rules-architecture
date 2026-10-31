plugins {
    alias(libs.plugins.klibs.gradle.detekt) apply false
    alias(libs.plugins.klibs.gradle.java.version) apply false
    alias(libs.plugins.klibs.gradle.publication) apply false
    alias(libs.plugins.klibs.gradle.rootinfo) apply false
    alias(libs.plugins.kotlin.jvm) apply false
    alias(libs.plugins.kover) apply false
}

val architectureSelfCheckSources = tasks.register<Copy>("architectureSelfCheckSources") {
    destinationDir = rootProject.file("self-check")
    listOf("core", "detekt", "gradle-plugin", "konsist", "structure").forEach { module ->
        from("$module/src") { into("$module/src") }
        from("$module/README.md") { into(module) }
    }
}

val architectureSelfCheck = tasks.register<Exec>("architectureSelfCheck") {
    group = "verification"
    dependsOn(architectureSelfCheckSources)
    dependsOn(
        ":core:publishToMavenLocal",
        ":detekt:publishToMavenLocal",
        ":gradle-plugin:publishToMavenLocal",
        ":konsist:publishToMavenLocal",
        ":structure:publishToMavenLocal"
    )
    workingDir = rootProject.file("self-check")
    commandLine(
        rootProject.file("gradlew").absolutePath,
        "-p",
        rootProject.file("self-check").absolutePath,
        "-DklibsRulesVersion=${project.property("klibs.project.version.string")}",
        "-DklibsKotlinVersion=${libs.versions.kotlin.version.get()}",
        ":architectureCheck",
        ":core:architectureCheck",
        ":detekt:architectureCheck",
        ":gradle-plugin:architectureCheck",
        ":konsist:architectureCheck",
        ":structure:architectureCheck",
        ":architectureGraphCheck"
    )
}

tasks.register("check") {
    group = "verification"
    dependsOn(architectureSelfCheck)
    dependsOn(":core:check", ":detekt:check", ":gradle-plugin:check", ":konsist:check", ":structure:check")
}
