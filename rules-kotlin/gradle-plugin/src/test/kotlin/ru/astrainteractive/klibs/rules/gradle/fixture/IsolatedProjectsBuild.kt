package ru.astrainteractive.klibs.rules.gradle.fixture

import org.gradle.testkit.runner.BuildResult
import org.gradle.testkit.runner.GradleRunner
import java.io.File

internal class IsolatedProjectsBuild(private val rootDirectory: File) {
    private val moduleNames = mutableListOf<String>()
    private var offline = true

    private fun writeFile(path: String, content: String) {
        val file = File(rootDirectory, path)
        file.parentFile.mkdirs()
        file.writeText(content)
    }

    fun file(path: String): File = File(rootDirectory, path)

    /**
     * Lets the build download dependencies, for a plugin that resolves a tool at execution time; the TestKit
     * Gradle home starts with an empty cache.
     */
    fun online(): IsolatedProjectsBuild {
        offline = false
        return this
    }

    fun withRootBuildScript(content: String): IsolatedProjectsBuild {
        writeFile("build.gradle.kts", content)
        return this
    }

    fun withModule(name: String, buildScript: String): IsolatedProjectsBuild {
        moduleNames.add(name)
        writeFile("$name/build.gradle.kts", buildScript)
        return this
    }

    fun withSourceFile(path: String, content: String): IsolatedProjectsBuild {
        writeFile(path, content)
        return this
    }

    private fun writeSettings() {
        val includes = moduleNames.joinToString(separator = "\n") { name -> """include(":$name")""" }
        writeFile(
            path = "settings.gradle.kts",
            content = """
                dependencyResolutionManagement {
                    repositories {
                        // The rules of the version under test are published here by the test task
                        mavenLocal {
                            content { includeGroup("ru.astrainteractive.klibs.rules") }
                        }
                        mavenCentral()
                    }
                }
                rootProject.name = "ip-fixture"
                $includes
            """.trimIndent()
        )
        writeFile(path = "gradle.properties", content = GRADLE_PROPERTIES)
    }

    private fun runner(arguments: Array<out String>): GradleRunner = GradleRunner.create()
        .withProjectDir(rootDirectory)
        .withPluginClasspath()
        .forwardOutput()
        .withArguments(arguments.toList() + listOfNotNull("--offline".takeIf { offline }, "--stacktrace"))

    fun run(vararg arguments: String): BuildResult {
        writeSettings()
        return runner(arguments).build()
    }

    fun runAndFail(vararg arguments: String): BuildResult {
        writeSettings()
        return runner(arguments).buildAndFail()
    }

    private companion object {
        val GRADLE_PROPERTIES = """
            org.gradle.isolated-projects=true
            org.gradle.configuration-cache=true
            org.gradle.parallel=true
        """.trimIndent()
    }
}
