package ru.astrainteractive.klibs.rules.declaration.detekt

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.Rule
import dev.detekt.psi.hasAnnotation
import org.jetbrains.kotlin.psi.KtClass
import org.jetbrains.kotlin.psi.KtClassOrObject
import org.jetbrains.kotlin.psi.KtFile
import org.jetbrains.kotlin.psi.KtNamedFunction
import org.jetbrains.kotlin.psi.KtUserType
import ru.astrainteractive.klibs.rules.detekt.util.isComponentDefaults

/**
 * Reports a file that holds more than one type family: a top-level function beside a top-level class, interface
 * or object (a composable beside a class included, since a composable file declares no class), and any type beside
 * another family. One sealed type and its variants form one family; unrelated data and value types do not. The
 * public `<Component>Defaults` object of a design-system component sits in its composable's file and is no family.
 */
internal class OneTypeFamilyPerFile(config: Config) : Rule(config, DESCRIPTION) {

    private fun superTypeNames(declaration: KtClassOrObject): List<String?> {
        return declaration.superTypeListEntries.map { entry ->
            (entry.typeReference?.typeElement as? KtUserType)?.referencedName
        }
    }

    private fun reportFunctions(functions: List<KtNamedFunction>, typeName: String?) {
        functions.forEach { function ->
            val message = if (function.hasAnnotation(COMPOSABLE)) {
                "'${function.name}' is a composable beside the type '$typeName': a composable file declares no " +
                    "class; move '$typeName' to its own file."
            } else {
                "'${function.name}' sits beside the type '$typeName': a file that declares a class holds no " +
                    "top-level function; make it a private member or an extension in util/<Subject>Ext.kt."
            }
            report(Finding(Entity.atName(function), message))
        }
    }

    override fun visitKtFile(file: KtFile) {
        super.visitKtFile(file)
        val types = file.declarations
            .filterIsInstance<KtClassOrObject>()
            .filterNot(KtClassOrObject::isComponentDefaults)
        val functions = file.declarations.filterIsInstance<KtNamedFunction>()
        if (types.isNotEmpty()) reportFunctions(functions, types.first().name)
        val sealedRoot = types.filterIsInstance<KtClass>().firstOrNull(KtClass::isSealed)
        types.forEachIndexed { index, declaration ->
            val isRoot = declaration === sealedRoot
            val isVariant = sealedRoot?.name in superTypeNames(declaration)
            val isSealedFamily = sealedRoot != null && (isRoot || isVariant)
            if (isSealedFamily || (sealedRoot == null && index == 0)) return@forEachIndexed
            val message = "Move '${declaration.name}' to its own file: a file holds one type family."
            report(Finding(Entity.atName(declaration), message))
        }
    }

    private companion object {
        const val COMPOSABLE = "Composable"
        const val DESCRIPTION = "A file holds one type family; a class with behaviour has its own file."
    }
}
