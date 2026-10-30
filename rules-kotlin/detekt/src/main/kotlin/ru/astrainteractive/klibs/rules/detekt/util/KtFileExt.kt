package ru.astrainteractive.klibs.rules.detekt.util

import org.jetbrains.kotlin.name.Name
import org.jetbrains.kotlin.psi.KtFile

private val TEST_SOURCE_SET = Regex("""(^|/)(test|testFixtures|[A-Za-z0-9]*Test)/""")

/**
 * Whether the file sits in a test source set: `test`, `testFixtures` or any `…Test` one (`commonTest`).
 */
internal fun KtFile.isTestSource(): Boolean = TEST_SOURCE_SET.containsMatchIn(virtualFilePath.replace('\\', '/'))

/**
 * Whether the file sits in a technology package (`exposed`, `ktor`, `room`, ...), where a library's own types and
 * calls are allowed.
 */
internal fun KtFile.isInTechnologyPackage(projectWords: List<String>): Boolean {
    return packageFqName.pathSegments()
        .map(Name::asString)
        .any { segment -> isTechnologySegment(segment, projectWords) }
}
