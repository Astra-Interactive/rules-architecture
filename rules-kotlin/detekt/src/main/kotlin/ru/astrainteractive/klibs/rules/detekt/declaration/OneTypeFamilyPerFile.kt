package ru.astrainteractive.klibs.rules.detekt.declaration

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
import org.jetbrains.kotlin.psi.psiUtil.collectDescendantsOfType

/**
 * Reports a file that holds more than one type family: a top-level function beside a top-level class, interface
 * or object (a composable beside a class included, since a composable file declares no class), and a second
 * top-level class with behaviour. A class has behaviour unless it is a data, value, enum, annotation or sealed
 * class, an interface, an object, or a direct subtype of a sealed type declared in the same file.
 */
internal class OneTypeFamilyPerFile(config: Config) : Rule(config, DESCRIPTION) {

    private fun superTypeNames(declaration: KtClass): List<String?> {
        return declaration.superTypeListEntries.map { entry ->
            (entry.typeReference?.typeElement as? KtUserType)?.referencedName
        }
    }

    private fun hasBehaviour(declaration: KtClass, sealedNames: Set<String?>): Boolean {
        val isDataHolder = declaration.isData() || declaration.isValue() || declaration.isInline()
        val isSpecial = declaration.isEnum() || declaration.isAnnotation() || declaration.isInterface()
        if (isDataHolder || isSpecial || declaration.isSealed()) return false
        return superTypeNames(declaration).none { name -> name in sealedNames }
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
        val types = file.declarations.filterIsInstance<KtClassOrObject>()
        val functions = file.declarations.filterIsInstance<KtNamedFunction>()
        if (types.isNotEmpty()) reportFunctions(functions, types.first().name)
        val sealedNames = file.collectDescendantsOfType<KtClass>(KtClass::isSealed)
            .map(KtClass::getName)
            .toSet()
        types.filterIsInstance<KtClass>()
            .filter { declaration -> hasBehaviour(declaration, sealedNames) }
            .drop(1)
            .forEach { declaration ->
                val message = "Move '${declaration.name}' to its own file: a class with behaviour always has its " +
                    "own file."
                report(Finding(Entity.atName(declaration), message))
            }
    }

    private companion object {
        const val COMPOSABLE = "Composable"
        const val DESCRIPTION = "A file holds one type family; a class with behaviour has its own file."
    }
}
