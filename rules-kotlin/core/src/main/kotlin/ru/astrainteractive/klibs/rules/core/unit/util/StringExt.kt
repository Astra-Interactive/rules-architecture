package ru.astrainteractive.klibs.rules.core.unit.util

import ru.astrainteractive.klibs.rules.core.unit.model.ProductKind
import ru.astrainteractive.klibs.rules.core.unit.model.UnitContext
import ru.astrainteractive.klibs.rules.core.unit.model.UnitKind
import java.io.ByteArrayInputStream
import java.io.DataInputStream
import java.util.Base64

public fun String.toUnitKindOrNull(): UnitKind? = UnitKind.entries.find { kind -> kind.cliName == this }

public fun String.toProductKindOrNull(): ProductKind? = ProductKind.entries.find { kind -> kind.name == this }

private fun readStrings(input: DataInputStream): Set<String> {
    return List(input.readInt()) { input.readUTF() }.toSet()
}

public fun String.toUnitContext(): UnitContext {
    val bytes = Base64.getDecoder().decode(this)
    return DataInputStream(ByteArrayInputStream(bytes)).use { input ->
        require(input.readInt() == 2) { "Unsupported unit context transport version" }
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
        val compilations = List(input.readInt()) {
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
        require(input.available() == 0) { "Trailing data in unit context transport" }
        UnitContext(
            kind,
            product,
            library,
            technologies,
            compilations
        )
    }
}
