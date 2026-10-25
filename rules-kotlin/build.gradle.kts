plugins {
    alias(libs.plugins.klibs.gradle.detekt) apply false
    alias(libs.plugins.klibs.gradle.java.version) apply false
    alias(libs.plugins.klibs.gradle.publication) apply false
    alias(libs.plugins.klibs.gradle.rootinfo) apply false
    alias(libs.plugins.kotlin.jvm) apply false
    alias(libs.plugins.kover) apply false
}

val architectureSelfCheckSources = tasks.register<Sync>("architectureSelfCheckSources") {
    destinationDir = rootProject.file("self-check/build/units")
    listOf("detekt", "gradle-plugin", "konsist", "structure").forEach { module ->
        from("$module/src") { into("$module/src") }
        from("$module/README.md") { into(module) }
        from("self-check/$module/build.gradle.kts") { into(module) }
        from("self-check/$module/architecture-baseline.txt") { into(module) }
    }
}

val architectureSelfCheck = tasks.register<GradleBuild>("architectureSelfCheck") {
    group = "verification"
    dependsOn(architectureSelfCheckSources)
    dependsOn(
        ":detekt:publishToMavenLocal",
        ":gradle-plugin:publishToMavenLocal",
        ":konsist:publishToMavenLocal",
        ":structure:publishToMavenLocal"
    )
    dir = rootProject.file("self-check")
    tasks = listOf(
        ":architectureCheck",
        ":detekt:architectureCheck",
        ":gradle-plugin:architectureCheck",
        ":konsist:architectureCheck",
        ":structure:architectureCheck",
        ":architectureGraphCheck"
    )
    startParameter.systemPropertiesArgs["klibsRulesVersion"] =
        project.property("klibs.project.version.string").toString()
    startParameter.systemPropertiesArgs["klibsKotlinVersion"] = libs.versions.kotlin.version.get()
}

tasks.register("check") {
    group = "verification"
    dependsOn(architectureSelfCheck)
    dependsOn(":detekt:check", ":gradle-plugin:check", ":konsist:check", ":structure:check")
}
