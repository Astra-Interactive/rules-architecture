plugins {
    id("org.jetbrains.kotlin.jvm")
    id("ru.astrainteractive.gradleplugin.detekt")
    id("ru.astrainteractive.gradleplugin.java.version")
    id("ru.astrainteractive.gradleplugin.publication")
    id("ru.astrainteractive.gradleplugin.rootinfo")
}

kotlin {
    explicitApi()
}

dependencies {
    compileOnly(libs.detekt.api)

    testImplementation(libs.detekt.test)
    testImplementation(libs.tests.kotlin.test)
    testImplementation(platform(libs.tests.junit.bom))
    testImplementation(libs.tests.junit.jupiter)
    testRuntimeOnly(libs.tests.junit.platform.launcher)
}

tasks.test {
    useJUnitPlatform()
}
