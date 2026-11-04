package ru.astrainteractive.klibs.rules.konsist.ownership.model

import ru.astrainteractive.klibs.rules.core.unit.model.UnitContext
import java.io.File

internal data class OwnershipUnit(
    val path: String,
    val directory: File,
    val context: UnitContext
)
