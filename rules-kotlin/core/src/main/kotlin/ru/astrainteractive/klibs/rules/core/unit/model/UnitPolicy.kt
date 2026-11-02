package ru.astrainteractive.klibs.rules.core.unit.model

public data class UnitPolicy(
    val composition: UiComposition,
    val implementationAccess: ImplementationAccess,
    val checks: CheckPolicy
)
