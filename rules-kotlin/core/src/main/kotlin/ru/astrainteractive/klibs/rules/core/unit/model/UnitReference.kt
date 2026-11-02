package ru.astrainteractive.klibs.rules.core.unit.model

/** Normalized reference; Gradle callers can obtain it from a typed project dependency. */
public data class UnitReference(public val path: String) {
    init {
        require(path.startsWith(":") && path != ":" && path.split(":").drop(1).all(String::isNotBlank)) {
            "Expected an absolute non-root Gradle project path, got '$path'"
        }
    }
}
