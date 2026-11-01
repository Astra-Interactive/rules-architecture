package ru.astrainteractive.klibs.rules.detekt.util

import dev.detekt.api.Config
import ru.astrainteractive.klibs.rules.core.unit.model.UnitContext
import ru.astrainteractive.klibs.rules.core.unit.util.toUnitContext

internal fun Config.unitContextOrNull(): UnitContext? {
    val encoded = valueOrNull<String>("unitContext")?.takeIf(String::isNotEmpty) ?: return null
    return encoded.toUnitContext()
}

internal fun Config.projectTechnologyWords(additional: List<String>): List<String> {
    return unitContextOrNull()?.technologyWords.orEmpty().plus(additional).distinct()
}
