package ru.astrainteractive.klibs.rules.konsist.check.model

import ru.astrainteractive.klibs.rules.konsist.layout.UnitKind
import java.io.File

/**
 * A build unit written to disk for a test: Kotlin files under `src/main/kotlin` and `src/test/kotlin`.
 */
internal class SourceTree(private val projectDirectory: File) {
    val mainDirectory: File = File(projectDirectory, "src/main/kotlin")
    val testDirectory: File = File(projectDirectory, "src/test/kotlin")

    private fun write(directory: File, packageName: String, fileName: String, code: String): SourceTree {
        val file = File(directory, "${packageName.replace('.', '/')}/$fileName")
        file.parentFile.mkdirs()
        val content = buildString {
            appendLine("package $packageName")
            appendLine()
            appendLine(code.trimIndent())
        }
        file.writeText(content)
        return this
    }

    fun main(packageName: String, fileName: String, code: String): SourceTree =
        write(mainDirectory, packageName, fileName, code)

    fun test(packageName: String, fileName: String, code: String): SourceTree =
        write(testDirectory, packageName, fileName, code)

    fun load(kind: UnitKind, rootPackage: String? = null): UnitSources = UnitSources.load(
        kind = kind,
        rootPackage = rootPackage,
        mainDirectories = listOf(mainDirectory),
        testDirectories = listOf(testDirectory),
        projectDirectory = projectDirectory
    )
}
