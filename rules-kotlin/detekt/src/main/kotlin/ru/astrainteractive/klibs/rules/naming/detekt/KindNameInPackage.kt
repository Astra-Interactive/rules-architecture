package ru.astrainteractive.klibs.rules.naming.detekt

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.RequiresAnalysisApi
import dev.detekt.api.Rule
import org.jetbrains.kotlin.analysis.api.analyze
import org.jetbrains.kotlin.name.ClassId
import org.jetbrains.kotlin.name.FqName
import org.jetbrains.kotlin.name.Name
import org.jetbrains.kotlin.psi.KtClassOrObject
import ru.astrainteractive.klibs.rules.core.unit.model.UnitKind
import ru.astrainteractive.klibs.rules.detekt.util.isTechnologySegment
import ru.astrainteractive.klibs.rules.detekt.util.projectTechnologyWords
import ru.astrainteractive.klibs.rules.detekt.util.unitContextOrNull
import dev.detekt.api.config as ruleConfig

/**
 * Reports a type name that carries a kind word its package does not allow:
 * - a resolved platform service outside `service`, except in an entry-point unit. Names alone never establish
 *   this role; consumers can extend the known base types and annotations for their framework;
 * - the `Model` or `Entity` suffix in a `model` package, unless a technology word (`exposed`, `room`, `table`, ...)
 *   precedes it in the package path, where a technology keeps its own representation;
 * - the `Helper`, `Util`, `Utils` and `Manager` suffixes anywhere: a class with behaviour has a role, and the role
 *   names it.
 */
internal class KindNameInPackage(config: Config) : Rule(config, DESCRIPTION), RequiresAnalysisApi {
    private val technologyWords: List<String> by ruleConfig(emptyList())
    private val serviceBaseTypes: List<String> by ruleConfig(emptyList())
    private val serviceAnnotations: List<String> by ruleConfig(emptyList())
    private val allTechnologyWords: List<String> by lazy { config.projectTechnologyWords(technologyWords) }

    private fun isDomainModelPackage(segments: List<String>): Boolean {
        val modelIndex = segments.indexOf(MODEL_SEGMENT)
        if (modelIndex < 0) return false
        return segments.take(modelIndex).none { segment -> isTechnologySegment(segment, allTechnologyWords) }
    }

    private fun serviceEvidence(classOrObject: KtClassOrObject): String? {
        return analyze(classOrObject) {
            val symbol = classOrObject.classSymbol ?: return@analyze null
            val type = symbol.defaultType
            val base = (SERVICE_BASE_TYPES + serviceBaseTypes).firstOrNull { name ->
                type.isSubtypeOf(ClassId.topLevel(FqName(name)))
            }
            if (base != null) return@analyze "subtype of $base"
            val annotations = symbol.annotations.mapNotNull { annotation ->
                annotation.classId?.asSingleFqName()?.asString()
            }
            (SERVICE_ANNOTATIONS + serviceAnnotations).firstOrNull(annotations::contains)?.let { name -> "@$name" }
        }
    }

    private fun serviceViolation(classOrObject: KtClassOrObject, name: String, segments: List<String>): String? {
        if (SERVICE_SEGMENT in segments) return null
        if (config.unitContextOrNull()?.kind == UnitKind.ENTRY_POINT) return null
        val evidence = serviceEvidence(classOrObject) ?: return null
        return "Place '$name' in a service package or the platform entry point: recognized as $evidence. " +
            "Service detection covers known types and annotations only; extend serviceBaseTypes/serviceAnnotations " +
            "with fully qualified names for another framework. An unrecognized type is not proven to be a domain value."
    }

    private fun findViolations(classOrObject: KtClassOrObject, name: String, segments: List<String>): List<String> {
        return buildList {
            ROLELESS_SUFFIXES
                .filter(name::endsWith)
                .forEach { suffix -> add("Name '$name' after its role instead of the '$suffix' suffix.") }
            serviceViolation(
                classOrObject = classOrObject,
                name = name,
                segments = segments
            )?.let(::add)
            if (isDomainModelPackage(segments)) {
                MODEL_SUFFIXES
                    .filter(name::endsWith)
                    .forEach { suffix -> add("Remove the '$suffix' suffix from the domain type '$name'.") }
            }
        }
    }

    override fun visitClassOrObject(classOrObject: KtClassOrObject) {
        super.visitClassOrObject(classOrObject)
        val name = classOrObject.name ?: return
        val segments = classOrObject.containingKtFile.packageFqName.pathSegments().map(Name::asString)
        findViolations(
            classOrObject = classOrObject,
            name = name,
            segments = segments
        ).forEach { message ->
            report(Finding(Entity.from(classOrObject), message))
        }
    }

    private companion object {
        const val SERVICE_SEGMENT = "service"
        const val MODEL_SEGMENT = "model"

        @JvmField
        val MODEL_SUFFIXES = listOf("Model", "Entity")

        @JvmField
        val SERVICE_BASE_TYPES = listOf(
            "android.app.Service",
            "ru.astrainteractive.astralibs.service.Service"
        )

        @JvmField
        val SERVICE_ANNOTATIONS = listOf("com.intellij.openapi.components.Service")

        @JvmField
        val ROLELESS_SUFFIXES = listOf(
            "Helper",
            "Util",
            "Utils",
            "Manager"
        )
        const val DESCRIPTION = "Kind words in a type name match the package that holds the type."
    }
}
