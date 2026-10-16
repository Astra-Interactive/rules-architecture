package ru.astrainteractive.klibs.rules.konsist.check.model

/**
 * One breach of an architecture rule.
 *
 * @property path the file, relative to the project directory, with `/` separators
 * @property subject the declaration, import or package the violation is about; together with [rule] and [path]
 * it identifies the violation in a baseline
 */
internal data class Violation(
    val rule: String,
    val path: String,
    val subject: String,
    val message: String
)
