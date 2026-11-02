package ru.astrainteractive.klibs.rules.detekt.util

import dev.detekt.api.Config
import ru.astrainteractive.klibs.rules.core.unit.model.UnitContext
import ru.astrainteractive.klibs.rules.core.unit.util.toUnitContext
import java.io.File

internal fun Config.unitContextOrNull(): UnitContext? {
    val contextPath = valueOrNull<String>("unitContextFile").orEmpty()
    if (contextPath.isNotEmpty()) return File(contextPath).readText().toUnitContext()
    val encoded = valueOrNull<String>("unitContext")?.takeIf(String::isNotEmpty) ?: return null
    return encoded.toUnitContext()
}

internal fun Config.projectTechnologyWords(additional: List<String>): List<String> {
    return unitContextOrNull()?.technologyWords.orEmpty().plus(additional).distinct()
}
