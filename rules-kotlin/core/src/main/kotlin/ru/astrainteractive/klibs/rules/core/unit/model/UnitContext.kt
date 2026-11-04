package ru.astrainteractive.klibs.rules.core.unit.model

public data class UnitContext(
    val kind: UnitKind,
    val productKind: ProductKind?,
    val publishedLibrary: Boolean,
    val technologyWords: Set<String>,
    val compilations: List<Compilation>,
    val policy: UnitPolicy
) {
    public constructor(
        kind: UnitKind,
        productKind: ProductKind?,
        publishedLibrary: Boolean,
        technologyWords: Set<String>,
        compilations: List<Compilation>
    ) : this(
        kind,
        productKind,
        publishedLibrary,
        technologyWords,
        compilations,
        UnitPolicy(
            UiComposition.LOCAL,
            if (kind == UnitKind.IMPL_UI) ImplementationAccess.SIBLINGS else ImplementationAccess.NONE,
            CheckPolicy(emptyMap(), emptyMap())
        )
    )

    /** [sourceSets] are visible through association; [compiledSourceSets] belong to this compilation. */
    public data class Compilation(
        val name: String,
        val sourceSets: Map<String, Set<String>>,
        val test: Boolean,
        val compose: Boolean?,
        val projectDependencies: Set<String>,
        val compiledSourceSets: Map<String, Set<String>>
    ) {
        public constructor(
            name: String,
            sourceSets: Map<String, Set<String>>,
            test: Boolean,
            compose: Boolean?,
            projectDependencies: Set<String>
        ) : this(name, sourceSets, test, compose, projectDependencies, sourceSets)
    }
}
