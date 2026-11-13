package ru.astrainteractive.klibs.rules.konsist.check.model

import ru.astrainteractive.klibs.rules.core.unit.model.UnitKind
import ru.astrainteractive.klibs.rules.konsist.layout.model.Kind
import ru.astrainteractive.klibs.rules.konsist.layout.model.Vocabulary
import java.io.File

/**
 * Writes the units of a worked tree to disk, each listed file as the smallest declaration its kind and name call for:
 * an interface in `api`, a `data class` in `model`, a `…UseCase` with one `operator fun invoke`, an extension of the
 * type a `util/<Type>Ext.kt` names, a `…Module` class in `di`, an implementation of the contract its name ends with in
 * a technology package, a class named as listed in an entry-point kind.
 */
internal class SkillTreeWriter(
    private val units: List<SkillUnit>,
    private val vocabulary: Vocabulary
) {
    private class Declaration(val imports: List<String>, val code: String)

    private val packageByType: Map<String, String> = units
        .flatMap { unit -> unit.files.map { file -> file.name to packageOf(unit, file) } }
        .toMap()

    private val contracts: Set<String> = units
        .flatMap { unit -> unit.files.filter { file -> kindOf(file) == Kind.CONTRACT }.map(SkillFile::name) }
        .toSet()

    private fun packageOf(unit: SkillUnit, file: SkillFile): String {
        return listOf(unit.rootPackage).plus(file.directory.split('/')).filter(String::isNotEmpty).joinToString(".")
    }

    private fun kindWordOf(file: SkillFile): String? {
        return file.directory.split('/').lastOrNull { segment -> Kind.fromSegment(segment, vocabulary) != null }
    }

    private fun kindOf(file: SkillFile): Kind? = kindWordOf(file)?.let { word -> Kind.fromSegment(word, vocabulary) }

    private fun importOf(type: String, packageName: String): List<String> {
        val declaring = packageByType[type] ?: LIBRARY_TYPES.getValue(type).substringBeforeLast('.')
        if (declaring == packageName) return emptyList()
        return listOf("$declaring.$type")
    }

    private fun visibilityIn(unit: SkillUnit): String = if (unit.kind in PUBLIC_UNIT_KINDS) "" else "internal "

    private fun joinLines(vararg lines: String): String = lines.joinToString("\n")

    private fun declareModel(unit: SkillUnit, file: SkillFile): Declaration {
        if (file.name.endsWith("Exception")) {
            return Declaration(
                emptyList(),
                "${visibilityIn(unit)}class ${file.name}(message: String) : IllegalStateException(message)"
            )
        }
        return Declaration(emptyList(), "${visibilityIn(unit)}data class ${file.name}(val value: Int)")
    }

    private fun declareUseCase(unit: SkillUnit, file: SkillFile): Declaration {
        if (INTERFACE_NOTE in file.note) {
            val code = joinLines(
                "interface ${file.name} {",
                "    operator fun invoke()",
                "}"
            )
            return Declaration(emptyList(), code)
        }
        if (file.name.startsWith(DEFAULT_PREFIX)) {
            val contract = file.name.removePrefix(DEFAULT_PREFIX)
            val code = joinLines(
                "internal class ${file.name} : $contract {",
                "    override operator fun invoke() = Unit",
                "}"
            )
            return Declaration(importOf(contract, packageOf(unit, file)), code)
        }
        val code = joinLines(
            "${visibilityIn(unit)}class ${file.name} {",
            "    operator fun invoke() = Unit",
            "}"
        )
        return Declaration(emptyList(), code)
    }

    private fun declareExtension(unit: SkillUnit, file: SkillFile): Declaration {
        val receiver = file.name.removeSuffix(EXTENSION_SUFFIX)
        val call = CALL_NOTE.find(file.note)
        val function = if (call == null) {
            "isValid(): Boolean = true"
        } else {
            val parameter = call.groupValues[2]
            "${call.groupValues[1]}($parameter: String): Boolean = $parameter.isNotEmpty()"
        }
        return Declaration(importOf(receiver, packageOf(unit, file)), "${visibilityIn(unit)}fun $receiver.$function")
    }

    private fun declareModule(unit: SkillUnit, file: SkillFile): Declaration {
        val hasUnitModule = unit.files.any { other -> other.directory == DI_WORD && other !== file }
        val isChild = file.directory != DI_WORD && hasUnitModule
        val isInternal = file.name == ROOT_MODULE || isChild
        return Declaration(emptyList(), "${if (isInternal) "internal " else ""}class ${file.name}")
    }

    private fun declareComposable(unit: SkillUnit, file: SkillFile): Declaration {
        val isTheme = kindOf(file) == Kind.THEME
        val parameters = if (isTheme) "content: @Composable () -> Unit" else ""
        val code = joinLines(
            "@Composable",
            "${visibilityIn(unit)}fun ${file.name}($parameters) {",
            if (isTheme) "    content.invoke()" else "",
            "}"
        )
        return Declaration(listOf(COMPOSABLE_IMPORT), code)
    }

    private fun declareEntryPoint(unit: SkillUnit, file: SkillFile): Declaration {
        return when (kindWordOf(file)) {
            COMPOSABLE_WORD -> declareComposable(unit, file)
            COMPONENT_WORD -> if (file.name.startsWith(DEFAULT_PREFIX)) {
                val contract = file.name.removePrefix(DEFAULT_PREFIX)
                Declaration(importOf(contract, packageOf(unit, file)), "internal class ${file.name} : $contract")
            } else {
                Declaration(emptyList(), "interface ${file.name}")
            }
            else -> Declaration(emptyList(), "internal class ${file.name}")
        }
    }

    private fun declareImplementation(unit: SkillUnit, file: SkillFile): Declaration {
        val packageName = packageOf(unit, file)
        if (unit.kind == UnitKind.ENTRY_POINT && file.directory.isEmpty()) {
            return Declaration(importOf(PLUGIN_CLASS, packageName), "class ${file.name} : $PLUGIN_CLASS()")
        }
        val word = kindWordOf(file)
        if (word == TABLE_WORD) {
            return Declaration(importOf(TABLE_CLASS, packageName), "internal object ${file.name} : $TABLE_CLASS()")
        }
        val contract = contracts.filter { name -> file.name.endsWith(name) && file.name != name }
            .maxByOrNull(String::length)
            ?: return Declaration(emptyList(), "internal class ${file.name}")
        val visibility = if (word in SHARED_IMPLEMENTATION_WORDS) "" else "internal "
        return Declaration(importOf(contract, packageName), "${visibility}class ${file.name} : $contract")
    }

    private fun declare(unit: SkillUnit, file: SkillFile): Declaration {
        return when (kindOf(file)) {
            Kind.MODEL -> declareModel(unit, file)
            Kind.CONTRACT -> Declaration(emptyList(), "${visibilityIn(unit)}interface ${file.name}")
            Kind.USECASE -> declareUseCase(unit, file)
            Kind.UTIL -> declareExtension(unit, file)
            Kind.DI -> declareModule(unit, file)
            Kind.THEME -> declareComposable(unit, file)
            Kind.ENTRY_POINT -> declareEntryPoint(unit, file)
            Kind.IMPLEMENTATION, null -> declareImplementation(unit, file)
            Kind.FAKE, Kind.ROLE -> error("The worked trees list no fake or role-kind file: ${file.path}")
        }
    }

    private fun writeUnit(unit: SkillUnit, directory: File) {
        unit.files.forEach { file ->
            val packageName = packageOf(unit, file)
            val declaration = declare(unit, file)
            val text = buildString {
                appendLine("package $packageName")
                appendLine()
                declaration.imports.forEach { name -> appendLine("import $name") }
                if (declaration.imports.isNotEmpty()) appendLine()
                appendLine(declaration.code)
            }
            val target = File(directory, "${packageName.replace('.', '/')}/${file.name}.kt")
            target.parentFile.mkdirs()
            target.writeText(text)
        }
    }

    /**
     * Writes [unit] to `<directory>/src/main/kotlin` and each unit of [dependencies] to its own
     * `<directory>/dependency/<path>/src/main/kotlin`.
     *
     * @return the main source directory followed by the dependencies' source directories
     */
    fun write(unit: SkillUnit, dependencies: List<SkillUnit>, directory: File): List<File> {
        val main = File(directory, MAIN_SOURCES)
        writeUnit(unit, main)
        val dependencyDirectories = dependencies.map { dependency ->
            File(directory, "dependency/${dependency.path}/$MAIN_SOURCES").also { sources ->
                writeUnit(dependency, sources)
            }
        }
        return listOf(main) + dependencyDirectories
    }

    private companion object {
        const val MAIN_SOURCES = "src/main/kotlin"
        const val DEFAULT_PREFIX = "Default"
        const val EXTENSION_SUFFIX = "Ext"
        const val INTERFACE_NOTE = "(interface"
        const val DI_WORD = "di"
        const val ROOT_MODULE = "RootModule"
        const val COMPOSABLE_WORD = "composable"
        const val COMPONENT_WORD = "component"
        const val TABLE_WORD = "table"
        const val PLUGIN_CLASS = "JavaPlugin"
        const val TABLE_CLASS = "Table"
        const val COMPOSABLE_IMPORT = "androidx.compose.runtime.Composable"

        @JvmField
        val CALL_NOTE = Regex("""^(\w+)\((\w+)\)""")

        @JvmField
        val PUBLIC_UNIT_KINDS = setOf(
            UnitKind.API,
            UnitKind.CORE_API,
            UnitKind.CORE_UI,
            UnitKind.STORAGE_CONTRACT
        )

        @JvmField
        val SHARED_IMPLEMENTATION_WORDS = setOf("memory", "stub")

        @JvmField
        val LIBRARY_TYPES = mapOf(
            "Button" to "ru.astrainteractive.astralibs.menu.clicker.Button",
            PLUGIN_CLASS to "org.bukkit.plugin.java.JavaPlugin",
            TABLE_CLASS to "org.jetbrains.exposed.sql.Table"
        )
    }
}
