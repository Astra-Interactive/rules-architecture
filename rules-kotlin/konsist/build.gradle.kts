import dev.detekt.gradle.Detekt

plugins {
    id("org.jetbrains.kotlin.jvm")
    id("org.jetbrains.kotlinx.kover")
    id("ru.astrainteractive.gradleplugin.java.version")
    id("ru.astrainteractive.gradleplugin.publication")
    id("ru.astrainteractive.gradleplugin.rootinfo")
    id("ru.astrainteractive.gradleplugin.detekt")
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

    implementation(libs.konsist)

    testImplementation(libs.tests.kotlin.test)
    testImplementation(platform(libs.tests.junit.bom))
    testImplementation(libs.tests.junit.jupiter)
    testRuntimeOnly(libs.tests.junit.platform.launcher)
}

tasks.withType<Detekt>().configureEach {
    config.from(isolated.rootProject.projectDirectory.file("detekt/src/main/resources/detekt-architecture.yml"))
    // detekt passes the explicit API mode only to its per-source-set tasks, not to the plain detekt task.
    freeCompilerArgs.add("-Xexplicit-api=strict")
}

tasks.test {
    useJUnitPlatform()
}
