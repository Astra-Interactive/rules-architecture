package ru.astrainteractive.klibs.rules.konsist.ownership.model

internal data class OwnershipDeclaration(
    val name: String,
    val unit: OwnershipUnit,
    val location: String,
    val references: Set<String>,
    val wiring: Boolean,
    val ambiguousReferences: Set<String>
)
