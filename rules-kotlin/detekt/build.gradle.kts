import dev.detekt.gradle.Detekt

plugins {
    id("org.jetbrains.kotlin.jvm")
    id("org.jetbrains.kotlinx.kover")
    id("ru.astrainteractive.gradleplugin.detekt")
    id("ru.astrainteractive.gradleplugin.java.version")
    id("ru.astrainteractive.gradleplugin.publication")
    id("ru.astrainteractive.gradleplugin.rootinfo")
}

kotlin {
    explicitApi()
}

kover {
    reports {
        verify {
            rule {
                minBound(100)
            }
        }
    }
}

dependencies {
    detektPlugins(projects.detekt)

    compileOnly(libs.detekt.api)
    compileOnly(libs.detekt.psi.utils)

    testImplementation(libs.detekt.core)
    testImplementation(libs.detekt.rules.coroutines)
    testImplementation(libs.detekt.rules.naming)
    testImplementation(libs.detekt.rules.style)
    testImplementation(libs.detekt.test)
    testImplementation(libs.tests.kotlin.test)
    testImplementation(platform(libs.tests.junit.bom))
    testImplementation(libs.tests.junit.jupiter)
    testRuntimeOnly(libs.detekt.psi.utils)
    testRuntimeOnly(libs.tests.junit.platform.launcher)
}

tasks.withType<Detekt>().configureEach {
    config.from(layout.projectDirectory.file("src/main/resources/detekt-architecture.yml"))
    // detekt passes the explicit API mode only to its per-source-set tasks, not to the plain detekt task.
    freeCompilerArgs.add("-Xexplicit-api=strict")
}

tasks.test {
    useJUnitPlatform()
}
