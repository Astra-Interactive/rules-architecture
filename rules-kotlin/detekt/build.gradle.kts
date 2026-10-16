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
    compileOnly(libs.detekt.api)
    compileOnly(libs.detekt.psi.utils)

    testImplementation(libs.detekt.test)
    testImplementation(libs.tests.kotlin.test)
    testImplementation(platform(libs.tests.junit.bom))
    testImplementation(libs.tests.junit.jupiter)
    testRuntimeOnly(libs.detekt.psi.utils)
    testRuntimeOnly(libs.tests.junit.platform.launcher)
}

tasks.test {
    useJUnitPlatform()
}
