package ru.astrainteractive.klibs.rules.detekt.naming

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.Rule
import org.jetbrains.kotlin.psi.KtClassOrObject

/**
 * Reports a type name that carries a kind word its package does not allow:
 * - `Service` outside a `service` package, the role of components the platform runs on its own lifecycle;
 * - the `Model` or `Entity` suffix in a `model` package, unless that package sits under `data`, where a
 *   technology keeps its own representation;
 * - the `Helper`, `Util`, `Utils` and `Manager` suffixes anywhere: a class with behaviour has a role, and the role
 *   names it.
 */
internal class KindNameInPackage(config: Config) : Rule(config, DESCRIPTION) {

    private fun isDomainModelPackage(segments: List<String>): Boolean {
        val modelIndex = segments.indexOf(MODEL_SEGMENT)
        if (modelIndex < 0) return false
        val dataIndex = segments.indexOf(DATA_SEGMENT)
        return dataIndex !in 0 until modelIndex
    }

    private fun findViolations(name: String, segments: List<String>): List<String> = buildList {
        ROLELESS_SUFFIXES
            .filter { suffix -> name.endsWith(suffix) }
            .forEach { suffix -> add("Name '$name' after its role instead of the '$suffix' suffix.") }
        if (name.contains(SERVICE_WORD) && SERVICE_SEGMENT !in segments) {
            add("Move '$name' to a '$SERVICE_SEGMENT' package, or drop '$SERVICE_WORD' from its name.")
        }
        if (isDomainModelPackage(segments)) {
            MODEL_SUFFIXES
                .filter { suffix -> name.endsWith(suffix) }
                .forEach { suffix -> add("Remove the '$suffix' suffix from the domain type '$name'.") }
        }
    }

    override fun visitClassOrObject(classOrObject: KtClassOrObject) {
        super.visitClassOrObject(classOrObject)
        val name = classOrObject.name ?: return
        val segments = classOrObject.containingKtFile.packageFqName.pathSegments().map { segment ->
            segment.asString()
        }
        findViolations(name, segments).forEach { message ->
            report(Finding(Entity.from(classOrObject), message))
        }
    }

    private companion object {
        const val SERVICE_WORD = "Service"
        const val SERVICE_SEGMENT = "service"
        const val MODEL_SEGMENT = "model"
        const val DATA_SEGMENT = "data"

        @JvmField
        val MODEL_SUFFIXES = listOf("Model", "Entity")

        @JvmField
        val ROLELESS_SUFFIXES = listOf("Helper", "Util", "Utils", "Manager")
        const val DESCRIPTION = "Kind words in a type name match the package that holds the type."
    }
}
