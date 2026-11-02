package ru.astrainteractive.klibs.rules.core.unit.util

import ru.astrainteractive.klibs.rules.core.unit.model.ImplementationAccess
import ru.astrainteractive.klibs.rules.core.unit.model.UiComposition
import ru.astrainteractive.klibs.rules.core.unit.model.UnitPolicy
import ru.astrainteractive.klibs.rules.core.unit.model.UnitRole

public fun UnitRole.toPolicy(): UnitPolicy {
    val composition = if (this is UnitRole.ImplUi) composition else UiComposition.LOCAL
    val access = if (this is UnitRole.ImplUi) implementationAccess else ImplementationAccess.NONE
    return UnitPolicy(
        composition,
        access,
        checks.toPolicy()
    )
}
