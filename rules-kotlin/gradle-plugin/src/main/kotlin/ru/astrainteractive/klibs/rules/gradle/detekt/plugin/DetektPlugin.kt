package ru.astrainteractive.klibs.rules.gradle.detekt.plugin

import dev.detekt.gradle.Detekt
import org.gradle.api.GradleException
import org.gradle.api.JavaVersion
import org.gradle.api.Plugin
import org.gradle.api.Project
import ru.astrainteractive.klibs.rules.gradle.detekt.extension.DetektRulesExtension
import ru.astrainteractive.klibs.rules.gradle.detekt.service.DetektConfigService
import ru.astrainteractive.klibs.rules.gradle.detekt.util.kotlinJvmTarget
import java.io.File

internal class DetektPlugin : Plugin<Project> {
    private fun readBundledConfig(): String {
        val stream = javaClass.classLoader.getResourceAsStream(DETEKT_CONFIG_FILE_NAME)
            ?: throw GradleException("$DETEKT_CONFIG_FILE_NAME is missing from the gradle plugin jar")
        return stream.use { input -> input.readBytes().decodeToString() }
    }

    private fun Project.detektConfigFile(): File = isolated.rootProject
        .projectDirectory
        .file("build/detekt/$DETEKT_CONFIG_FILE_NAME")
        .asFile

    override fun apply(target: Project) {
        target.pluginManager.apply("dev.detekt")

        val configService = target.gradle.sharedServices.registerIfAbsent(
            CONFIG_SERVICE_NAME,
            DetektConfigService::class.java
        ) { service ->
            service.parameters.content.set(target.provider { readBundledConfig() })
            service.parameters.configFile.fileValue(target.detektConfigFile())
        }
        val detektConfigFile = configService.map { service -> service.configFile }
        val extension = target.extensions.create(EXTENSION_NAME, DetektRulesExtension::class.java)
        extension.jvmTarget.convention(
            target.provider { target.kotlinJvmTarget() ?: JavaVersion.current().majorVersion }
        )
        val projectSource = target.files(target.projectDir)

        target.tasks.register("detektFormat", Detekt::class.java) { task ->
            task.autoCorrect.set(true)
        }

        target.tasks.withType(Detekt::class.java).configureEach { task ->
            task.outputs.upToDateWhen { false }

            task.reports { reports ->
                reports.html.required.set(true)
                reports.checkstyle.required.set(false)
            }

            task.usesService(configService)
            task.config.setFrom(detektConfigFile)
            task.setSource(projectSource)

            task.include("**/*.kt", "**/*.kts")
            task.exclude(
                "**/resources/**",
                "**/build/**",
            )

            task.parallel.set(true)

            task.buildUponDefaultConfig.set(true)

            task.allRules.set(true)

            task.jvmTarget.set(extension.jvmTarget)
        }

        target.dependencies.add("detektPlugins", "dev.detekt:detekt-rules-ktlint-wrapper:$KTLINT_WRAPPER_VERSION")
        target.dependencies.add("detektPlugins", "io.nlopez.compose.rules:detekt:$COMPOSE_RULES_VERSION")
    }

    private companion object {
        const val DETEKT_CONFIG_FILE_NAME = "detekt.yml"
        const val CONFIG_SERVICE_NAME = "klibsDetektConfig"
        const val EXTENSION_NAME = "detektRules"
        const val KTLINT_WRAPPER_VERSION = "2.0.0-alpha.2"
        const val COMPOSE_RULES_VERSION = "0.5.6"
    }
}
