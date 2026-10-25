package ru.astrainteractive.klibs.rules.structure.unit.model

/**
 * One breach of a structure rule.
 *
 * @property path the file or directory the violation is about, relative to the unit's directory
 * @property subject what the violation is about (a dependency, a directory); with [rule] and [path] it identifies
 * the violation in a baseline
 */
public data class StructureViolation(
    val rule: String,
    val path: String,
    val subject: String,
    val message: String
)
