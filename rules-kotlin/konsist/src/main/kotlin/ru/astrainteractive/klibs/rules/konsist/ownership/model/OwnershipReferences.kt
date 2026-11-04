package ru.astrainteractive.klibs.rules.konsist.ownership.model

internal data class OwnershipReferences(
    val names: Set<String>,
    val ambiguous: Set<String>
)
