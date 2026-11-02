package ru.astrainteractive.klibs.rules.core.unit.util

import ru.astrainteractive.klibs.rules.core.unit.model.CheckPolicy
import ru.astrainteractive.klibs.rules.core.unit.model.ImplementationAccess
import ru.astrainteractive.klibs.rules.core.unit.model.ProductKind
import ru.astrainteractive.klibs.rules.core.unit.model.UiComposition
import ru.astrainteractive.klibs.rules.core.unit.model.UnitContext
import ru.astrainteractive.klibs.rules.core.unit.model.UnitKind
import ru.astrainteractive.klibs.rules.core.unit.model.UnitPolicy
import java.io.ByteArrayInputStream
import java.io.DataInputStream
import java.io.IOException
import java.util.Base64

private const val LEGACY_UNIT_CONTEXT_TRANSPORT_VERSION = 2

public fun String.toUnitKindOrNull(): UnitKind? = UnitKind.entries.find { kind -> kind.cliName == this }

public fun String.toProductKindOrNull(): ProductKind? = ProductKind.entries.find { kind -> kind.name == this }

private fun readStrings(input: DataInputStream): Set<String> {
    return List(input.readInt()) { input.readUTF() }.toSet()
}

private fun readReasons(input: DataInputStream): Map<String, String> {
    val count = input.readInt()
    require(count >= 0) { "Negative reason count in unit context transport" }
    return buildMap {
        repeat(count) {
            val target = input.readUTF()
            val reason = input.readUTF()
            require(target.isNotBlank()) { "Blank policy target in unit context transport" }
            require(reason.isNotBlank()) { "Blank policy reason for '$target'" }
            require(put(target, reason) == null) { "Duplicate policy target '$target'" }
        }
    }
}

private fun readPolicy(input: DataInputStream): UnitPolicy {
    val compositionName = input.readUTF()
    val composition = UiComposition.entries.find { mode -> mode.name == compositionName }
    requireNotNull(composition) { "Unknown UI composition '$compositionName'" }
    val accessName = input.readUTF()
    val access = ImplementationAccess.entries.find { mode -> mode.name == accessName }
    requireNotNull(access) { "Unknown implementation access '$accessName'" }
    return UnitPolicy(
        composition,
        access,
        CheckPolicy(readReasons(input), readReasons(input))
    )
}

private fun readCompilations(input: DataInputStream): List<UnitContext.Compilation> {
    return List(input.readInt()) {
        val name = input.readUTF()
        val test = input.readBoolean()
        val compose = when (val capability = input.readUTF()) {
            "" -> null
            "true" -> true
            "false" -> false
            else -> error("Unknown Compose capability '$capability'")
        }
        val dependencies = readStrings(input)
        val sourceSets = buildMap {
            repeat(input.readInt()) {
                put(input.readUTF(), readStrings(input))
            }
        }
        UnitContext.Compilation(
            name,
            sourceSets,
            test,
            compose,
            dependencies
        )
    }
}

public fun String.toUnitContext(): UnitContext {
    val bytes = Base64.getDecoder().decode(this)
    return try {
        DataInputStream(ByteArrayInputStream(bytes)).use { input ->
            val version = input.readInt()
            val isLegacy = version == LEGACY_UNIT_CONTEXT_TRANSPORT_VERSION
            val isCurrent = version == UNIT_CONTEXT_TRANSPORT_VERSION
            require(isLegacy || isCurrent) { "Unsupported unit context transport version $version" }
            val kindName = input.readUTF()
            val kind = requireNotNull(kindName.toUnitKindOrNull()) { "Unknown unit kind '$kindName'" }
            val productName = input.readUTF()
            val product = if (productName.isEmpty()) {
                null
            } else {
                requireNotNull(productName.toProductKindOrNull()) { "Unknown product kind '$productName'" }
            }
            val library = input.readBoolean()
            val technologies = readStrings(input)
            val compilations = readCompilations(input)
            val context = if (isCurrent) {
                UnitContext(
                    kind,
                    product,
                    library,
                    technologies,
                    compilations,
                    readPolicy(input)
                )
            } else {
                UnitContext(
                    kind,
                    product,
                    library,
                    technologies,
                    compilations
                )
            }
            require(input.available() == 0) { "Trailing data in unit context transport" }
            context
        }
    } catch (exception: IOException) {
        throw IllegalArgumentException("Corrupt or truncated unit context transport", exception)
    }
}
