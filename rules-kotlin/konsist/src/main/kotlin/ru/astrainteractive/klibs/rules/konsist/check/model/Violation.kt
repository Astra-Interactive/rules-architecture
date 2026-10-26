package ru.astrainteractive.klibs.rules.konsist.check.model

/**
 * One breach of an architecture rule.
 *
 * @property subject the declaration, import or package the violation is about; together with [rule] and [path]
 * it identifies the violation in a baseline
 * @property origin the source declaration, file or external path that produced the violation
 */
internal data class Violation(
    val rule: String,
    val subject: String,
    val message: String,
    val origin: ViolationOrigin
) {
    val path: String
        get() = origin.path

    constructor(rule: String, path: String, subject: String, message: String) : this(
        rule,
        subject,
        message,
        ExternalOrigin(path)
    )
}
