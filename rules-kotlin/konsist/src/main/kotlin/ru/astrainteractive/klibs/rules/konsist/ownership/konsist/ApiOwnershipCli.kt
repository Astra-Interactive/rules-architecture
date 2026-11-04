@file:JvmName("ApiOwnershipCli")

package ru.astrainteractive.klibs.rules.konsist.ownership.konsist

import ru.astrainteractive.klibs.rules.core.unit.util.toUnitContext
import ru.astrainteractive.klibs.rules.konsist.ownership.model.OwnershipUnit
import java.io.File

private const val METADATA_COLUMNS = 3

/** Writes an advisory API ownership report from a Gradle metadata file to the given output file. */
public fun main(args: Array<String>) {
    require(args.size == 2) { "Expected metadata file and report file" }
    val units = File(args[0]).readLines().filter(String::isNotBlank).map { line ->
        val parts = line.split('\t')
        require(parts.size == METADATA_COLUMNS) { "Expected unit path, directory and encoded context" }
        OwnershipUnit(
            parts[0],
            File(parts[1]),
            parts[2].toUnitContext()
        )
    }
    val output = File(args[1])
    output.parentFile.mkdirs()
    output.writeText(KonsistOwnershipReport().render(units))
}
