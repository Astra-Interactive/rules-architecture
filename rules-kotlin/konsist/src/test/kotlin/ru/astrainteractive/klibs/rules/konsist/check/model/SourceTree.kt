package ru.astrainteractive.klibs.rules.konsist.check.model

import ru.astrainteractive.klibs.rules.konsist.check.konsist.UnitSources
import ru.astrainteractive.klibs.rules.konsist.layout.model.UnitKind
import ru.astrainteractive.klibs.rules.konsist.layout.model.Vocabulary
import java.io.File

/**
 * A build unit written to disk for a test: Kotlin files under `src/main/kotlin` and `src/test/kotlin`.
 */
internal class SourceTree(
    private val projectDirectory: File,
    private val androidNamespace: String? = null
) {
    val mainDirectory: File = File(projectDirectory, "src/main/kotlin")
    val testDirectory: File = File(projectDirectory, "src/test/kotlin")
    private val dependencyDirectory: File = File(projectDirectory, "dependency/src/main/kotlin")

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

    fun main(packageName: String, fileName: String, code: String): SourceTree {
        return write(
            mainDirectory,
            packageName,
            fileName,
            code
        )
    }

    /**
     * Writes [code] to a unit this one depends on, at [path] written as for [main].
     */
    fun dependency(path: String, code: String): SourceTree {
        return write(
            dependencyDirectory,
            path.substringBefore('/'),
            path.substringAfter('/'),
            code
        )
    }

    fun test(packageName: String, fileName: String, code: String): SourceTree {
        return write(
            testDirectory,
            packageName,
            fileName,
            code
        )
    }

    /**
     * Writes [code] to [path], a package and a file name joined by a slash: `com.example.jail.model/Jail.kt`.
     */
    fun main(path: String, code: String): SourceTree {
        return main(
            packageName = path.substringBefore('/'),
            fileName = path.substringAfter('/'),
            code = code
        )
    }

    fun test(path: String, code: String): SourceTree {
        return test(
            packageName = path.substringBefore('/'),
            fileName = path.substringAfter('/'),
            code = code
        )
    }

    fun main(sourceSet: String, packageName: String, fileName: String, code: String): SourceTree {
        return write(
            File(projectDirectory, "src/$sourceSet/kotlin"),
            packageName,
            fileName,
            code
        )
    }

    fun test(sourceSet: String, packageName: String, fileName: String, code: String): SourceTree {
        return write(
            File(projectDirectory, "src/$sourceSet/kotlin"),
            packageName,
            fileName,
            code
        )
    }

    fun loadSourceSets(kind: UnitKind, mainSourceSet: String, testSourceSet: String): UnitSources {
        val vocabulary = NO_PROJECT_WORDS
        val main = File(projectDirectory, "src/$mainSourceSet/kotlin")
        val test = File(projectDirectory, "src/$testSourceSet/kotlin")
        return UnitSources.load(
            kind = kind,
            rootPackage = null,
            unitPath = null,
            mainDirectories = listOf(main),
            testDirectories = listOf(test),
            mainSourceSets = mapOf(mainSourceSet to main),
            testSourceSets = mapOf(testSourceSet to test),
            projectDirectory = projectDirectory,
            vocabulary = vocabulary,
            dependencyDirectories = listOf(dependencyDirectory),
            androidNamespace = androidNamespace
        )
    }

    fun load(
        kind: UnitKind,
        rootPackage: String? = null,
        unitPath: String? = null,
        vocabulary: Vocabulary = NO_PROJECT_WORDS
    ): UnitSources {
        return UnitSources.load(
            kind = kind,
            rootPackage = rootPackage,
            unitPath = unitPath,
            mainDirectories = listOf(mainDirectory),
            testDirectories = listOf(testDirectory),
            mainSourceSets = mapOf("main" to mainDirectory),
            testSourceSets = mapOf("test" to testDirectory),
            projectDirectory = projectDirectory,
            vocabulary = vocabulary,
            dependencyDirectories = listOf(dependencyDirectory),
            androidNamespace = androidNamespace
        )
    }

    companion object {
        val NO_PROJECT_WORDS = Vocabulary(
            entryPointWords = emptySet(),
            technologyWords = emptySet(),
            pluralConcepts = emptySet(),
            platformClasses = emptySet()
        )
    }
}
