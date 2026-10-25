import dev.detekt.gradle.Detekt

plugins {
    id("org.jetbrains.kotlin.jvm")
    id("java-gradle-plugin")
    id("org.jetbrains.kotlinx.kover")
    id("ru.astrainteractive.gradleplugin.java.version")
    id("ru.astrainteractive.gradleplugin.publication")
    id("ru.astrainteractive.gradleplugin.rootinfo")
    id("ru.astrainteractive.gradleplugin.detekt")
}

kotlin {
    explicitApi()
}

// TestKit runs out of process, so kover sees only the models
kover {
    reports {
        filters {
            excludes {
                classes(
                    "*.plugin.*",
                    "*.task.*",
                    "*.extension.*",
                    "*.service.*",
                    "*.util.*"
                )
            }
        }
        verify {
            rule {
                minBound(100)
            }
        }
    }
}

val testPluginDependencies = configurations.dependencyScope("testPluginDependencies")
val testPluginClasspath = configurations.resolvable("testPluginClasspath") {
    extendsFrom(testPluginDependencies.get())
}

dependencies {
    detektPlugins(projects.detekt)

    compileOnly(libs.kotlin.gradle)

    implementation(libs.detekt.gradle)
    implementation(projects.structure)
    // The linter reads detekt-architecture.yml from the rules jar
    runtimeOnly(projects.detekt)

    testImplementation(gradleTestKit())
    testImplementation(libs.tests.kotlin.test)
    testImplementation(platform(libs.tests.junit.bom))
    testImplementation(libs.tests.junit.jupiter)
    testRuntimeOnly(libs.tests.junit.platform.launcher)

    // Kotlin Gradle plugin is compileOnly, so the tests add it back
    add(testPluginDependencies.name, libs.kotlin.gradle)
}

// The linter resolves the rules at its own version
val generateRulesVersion = tasks.register<WriteProperties>("generateRulesVersion") {
    destinationFile = layout.buildDirectory.file("generated/klibs-rules/klibs-rules.properties")
    property("version", project.version.toString())
}

sourceSets.main {
    resources.srcDir(generateRulesVersion.map { task -> task.destinationFile.get().asFile.parentFile })
}

gradlePlugin {
    val projectUrl = providers.gradleProperty("klibs.project.url")
    website.set(projectUrl)
    vcsUrl.set(projectUrl)
    plugins {
        create("detekt") {
            id = "ru.astrainteractive.klibs.rules.gradle.detekt"
            implementationClass = "ru.astrainteractive.klibs.rules.gradle.detekt.plugin.gradle.DetektPlugin"
            displayName = "Detekt Code Analysis Plugin"
            description =
                "Automatically applies detekt with KLibs configuration for static code analysis with Kotlin linting rules"
            tags.set(
                listOf(
                    "kotlin",
                    "static-analysis",
                    "klibs"
                )
            )
        }
        create("linter") {
            id = "ru.astrainteractive.klibs.rules.gradle.linter"
            implementationClass = "ru.astrainteractive.klibs.rules.gradle.linter.plugin.gradle.LinterPlugin"
            displayName = "Architecture Linter Plugin"
            description = "Applies detekt with the architecture rules and checks the package layout and unit graph"
            tags.set(
                listOf(
                    "kotlin",
                    "static-analysis",
                    "architecture",
                    "klibs"
                )
            )
        }
    }
}

tasks.pluginUnderTestMetadata {
    pluginClasspath.from(testPluginClasspath)
}

tasks.withType<Detekt>().configureEach {
    config.from(isolated.rootProject.projectDirectory.file("detekt/src/main/resources/detekt-architecture.yml"))
    // detekt passes the explicit API mode only to its per-source-set tasks, not to the plain detekt task.
    freeCompilerArgs.add("-Xexplicit-api=strict")
}

tasks.test {
    useJUnitPlatform()
    // The tests resolve the rules from Maven Local: this version is not on Central yet
    dependsOn(
        ":detekt:publishToMavenLocal",
        ":konsist:publishToMavenLocal",
        ":structure:publishToMavenLocal"
    )
}
