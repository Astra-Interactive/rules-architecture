package ru.astrainteractive.klibs.rules.konsist.check.declaration.konsist

import com.lemonappdev.konsist.api.declaration.KoFileDeclaration
import com.lemonappdev.konsist.api.provider.KoNameProvider
import ru.astrainteractive.klibs.rules.konsist.check.konsist.ArchitectureCheck
import ru.astrainteractive.klibs.rules.konsist.check.konsist.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.layout.model.UnitKind

/**
 * Reports an anonymous `object :` that implements a type of this unit or of a unit it depends on: an anonymous
 * object only implements a type the project does not declare, a library's listener in a bridge; a type of the
 * project gets a named class of its own file.
 */
internal class AnonymousProjectObject : ArchitectureCheck {
    override val unitKinds: Set<UnitKind> = UnitKind.entries.toSet()

    private fun declaredTypeNames(files: List<KoFileDeclaration>): Set<String> {
        return files.flatMap { file ->
            file.classes(includeNested = true, includeLocal = false).map(KoNameProvider::name) +
                file.interfaces(includeNested = true).map(KoNameProvider::name)
        }.toSet()
    }

    override fun check(sources: UnitSources): List<Violation> {
        val projectTypes = declaredTypeNames(sources.mainFiles + sources.dependencyFiles)
        return sources.mainFiles.flatMap { file ->
            val code = file.text.replace(NON_CODE, "")
            ANONYMOUS_OBJECT.findAll(code)
                .map { match -> match.groupValues[1] }
                .filter { name -> name in projectTypes }
                .distinct()
                .map { name ->
                    val message = "Write a named class for the anonymous object : $name: an anonymous object only " +
                        "implements a type the project does not declare, such as a library's listener."
                    sources.createViolation(
                        RULE,
                        file,
                        name,
                        message
                    )
                }
                .toList()
        }
    }

    private companion object {
        const val RULE = "AnonymousProjectObject"

        @JvmField
        val ANONYMOUS_OBJECT = Regex("""(?<![\w.])(?<!companion )object\s*:\s*([A-Z]\w*)""")

        @JvmField
        val NON_CODE = Regex(""""(?:\\.|[^"\\])*"|//[^\n]*|/\*[\s\S]*?\*/""")
    }
}
