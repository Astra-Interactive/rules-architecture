package ru.astrainteractive.klibs.rules.core.unit.util

import ru.astrainteractive.klibs.rules.core.unit.model.UnitContext
import java.io.ByteArrayOutputStream
import java.io.DataOutputStream
import java.io.File
import java.util.Base64

internal const val UNIT_CONTEXT_TRANSPORT_VERSION = 4

private fun writeStrings(output: DataOutputStream, values: Collection<String>) {
    output.writeInt(values.size)
    values.sorted().forEach(output::writeUTF)
}

private fun writeReasons(output: DataOutputStream, reasons: Map<String, String>) {
    output.writeInt(reasons.size)
    reasons.toSortedMap().forEach { (target, reason) ->
        output.writeUTF(target)
        output.writeUTF(reason)
    }
}

/**
 * The same versioned transport is used by the Konsist CLI, generated Detekt settings and Gradle task inputs.
 */
public fun UnitContext.encode(): String {
    val bytes = ByteArrayOutputStream()
    DataOutputStream(bytes).use { output ->
        output.writeInt(UNIT_CONTEXT_TRANSPORT_VERSION)
        output.writeUTF(kind.cliName)
        output.writeUTF(productKind?.name.orEmpty())
        output.writeBoolean(publishedLibrary)
        writeStrings(output, technologyWords)
        output.writeInt(compilations.size)
        compilations.sortedBy(UnitContext.Compilation::name).forEach { compilation ->
            output.writeUTF(compilation.name)
            output.writeBoolean(compilation.test)
            output.writeUTF(compilation.compose?.toString().orEmpty())
            writeStrings(output, compilation.projectDependencies)
            output.writeInt(compilation.sourceSets.size)
            compilation.sourceSets.toSortedMap().forEach { (name, directories) ->
                output.writeUTF(name)
                writeStrings(output, directories)
            }
            output.writeInt(compilation.compiledSourceSets.size)
            compilation.compiledSourceSets.toSortedMap().forEach { (name, directories) ->
                output.writeUTF(name)
                writeStrings(output, directories)
            }
        }
        output.writeUTF(policy.composition.name)
        output.writeUTF(policy.implementationAccess.name)
        writeReasons(output, policy.checks.disabledRules)
        writeReasons(output, policy.checks.allowedDependencies)
    }
    return Base64.getEncoder().encodeToString(bytes.toByteArray())
}

/** Compilations that compile [file], excluding source roots visible only through association. */
public fun UnitContext.compilationsFor(file: File): List<UnitContext.Compilation> {
    val path = file.toPath().toAbsolutePath().normalize()
    return compilations.filter { compilation ->
        compilation.compiledSourceSets.values.flatten().any { directory ->
            path.startsWith(File(directory).toPath().toAbsolutePath().normalize())
        }
    }
}

/**
 * Unknown membership stays unknown; a shared source must have Compose in every compilation that uses it.
 */
public fun UnitContext.isCompose(file: File): Boolean? {
    val selected = compilationsFor(file)
    if (selected.isEmpty()) return null
    val capabilities = selected.map(UnitContext.Compilation::compose)
    if (false in capabilities) return false
    if (null in capabilities) return null
    return true
}
