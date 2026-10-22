package ru.astrainteractive.klibs.rules.detekt.util

import org.jetbrains.kotlin.psi.KtFile

private val TEST_SOURCE_SET = Regex("""(^|/)(test|testFixtures|[A-Za-z0-9]*Test)/""")

/**
 * Whether the file sits in a test source set: `test`, `testFixtures` or any `…Test` one (`commonTest`).
 */
internal fun KtFile.isTestSource(): Boolean = TEST_SOURCE_SET.containsMatchIn(virtualFilePath.replace('\\', '/'))
