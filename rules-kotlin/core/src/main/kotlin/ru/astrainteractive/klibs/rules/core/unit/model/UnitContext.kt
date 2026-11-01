package ru.astrainteractive.klibs.rules.core.unit.model

public data class UnitContext(
    val kind: UnitKind,
    val productKind: ProductKind?,
    val publishedLibrary: Boolean,
    val technologyWords: Set<String>,
    val compilations: List<Compilation>
) {
    public data class Compilation(
        val name: String,
        val sourceSets: Map<String, Set<String>>,
        val test: Boolean,
        val compose: Boolean,
        val projectDependencies: Set<String>
    )
}
