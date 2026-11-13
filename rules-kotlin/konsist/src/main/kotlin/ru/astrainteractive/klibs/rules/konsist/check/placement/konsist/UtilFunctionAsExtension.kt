package ru.astrainteractive.klibs.rules.konsist.check.placement.konsist

import com.lemonappdev.konsist.api.declaration.KoFileDeclaration
import com.lemonappdev.konsist.api.declaration.KoFunctionDeclaration
import ru.astrainteractive.klibs.rules.core.unit.model.UnitKind
import ru.astrainteractive.klibs.rules.konsist.check.konsist.ArchitectureCheck
import ru.astrainteractive.klibs.rules.konsist.check.konsist.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.check.model.util.simpleName
import ru.astrainteractive.klibs.rules.konsist.check.model.util.topLevelTypeNames
import ru.astrainteractive.klibs.rules.konsist.layout.model.Kind

/**
 * Reports a `util` function whose form does not follow its signature, judged over the models of the unit and of the
 * units it depends on (`AstraRatingConfig` of `core`), a typed identifier (`…Id`) not counted. A function that
 * returns the type of a model parameter (`T`, `T?`, `Result<T>`), or a top-level one whose models are all of one
 * type, is an extension of that type; an extension of a model that returns another type and takes a model of
 * another type is a top-level function named after the decision. A factory (`create<Type>`) and `parse<Type>` keep
 * their form.
 */
internal class UtilFunctionAsExtension : ArchitectureCheck {
    override val unitKinds: Set<UnitKind> = UnitKind.entries.toSet()

    private fun findModelNames(sources: UnitSources): Set<String> {
        return sources.mainFiles
            .filter { file -> sources.placementOf(file)?.kind == Kind.MODEL }
            .plus(sources.dependencyModelFiles)
            .flatMap(KoFileDeclaration::topLevelTypeNames)
            .filterNot { name -> name.endsWith(IDENTIFIER_SUFFIX) }
            .toSet()
    }

    private fun findReturnedName(function: KoFunctionDeclaration): String? {
        val text = function.returnType?.text?.removeSuffix("?") ?: return null
        val returned = RESULT_TYPE.matchEntire(text)?.groupValues?.get(1) ?: text
        return returned.removeSuffix("?").substringBefore('<').substringAfterLast('.')
    }

    private fun findModelParameters(function: KoFunctionDeclaration, modelNames: Set<String>): List<String> {
        return function.parameters.map { parameter -> parameter.type.simpleName }.filter(modelNames::contains)
    }

    private fun createTransitionMessage(function: KoFunctionDeclaration, returned: String): String {
        return "Write ${function.name} as an extension of $returned: fun $returned.${function.name}(), a " +
            "transition is an extension of the type it returns."
    }

    private fun findTopLevelMessage(function: KoFunctionDeclaration, modelNames: Set<String>): String? {
        val models = findModelParameters(function, modelNames)
        val returned = findReturnedName(function)
        if (returned != null && returned in models) return createTransitionMessage(function, returned)
        val model = models.distinct().singleOrNull() ?: return null
        return "Write ${function.name} as an extension of $model: fun $model.${function.name}(), a rule about one " +
            "type extends that type."
    }

    private fun findExtensionMessage(
        function: KoFunctionDeclaration,
        receiver: String,
        modelNames: Set<String>
    ): String? {
        val returned = findReturnedName(function)
        if (receiver !in modelNames || returned == receiver) return null
        val models = findModelParameters(function, modelNames)
        if (returned != null && returned in models) return createTransitionMessage(function, returned)
        val others = models.filterNot { model -> model == receiver }.distinct()
        if (others.isEmpty()) return null
        val read = listOf(receiver).plus(others).joinToString(" and ")
        return "Write ${function.name} as a top-level function of $read in a file named after the decision: a rule " +
            "that reads two model types is a top-level function."
    }

    override fun check(sources: UnitSources): List<Violation> {
        val modelNames = findModelNames(sources)
        return sources.mainFiles
            .filter { file -> sources.placementOf(file)?.kind == Kind.UTIL }
            .flatMap { file ->
                file.functions(includeNested = false, includeLocal = false)
                    .filterNot { function -> FACTORY_NAME.matches(function.name) }
                    .mapNotNull { function ->
                        val receiver = function.receiverType?.simpleName
                        val message = if (receiver == null) {
                            findTopLevelMessage(function, modelNames)
                        } else {
                            findExtensionMessage(
                                function,
                                receiver,
                                modelNames
                            )
                        } ?: return@mapNotNull null
                        val subject = if (receiver == null) function.name else "$receiver.${function.name}"
                        sources.createViolation(
                            RULE,
                            file,
                            function,
                            subject,
                            message
                        )
                    }
            }
    }

    private companion object {
        const val RULE = "UtilFunctionAsExtension"
        const val IDENTIFIER_SUFFIX = "Id"

        @JvmField
        val FACTORY_NAME = Regex("""(?:create|parse)[A-Z]\w*""")

        @JvmField
        val RESULT_TYPE = Regex("""(?:kotlin\.)?Result<(.+)>""")
    }
}
