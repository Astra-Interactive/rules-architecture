package ru.astrainteractive.klibs.rules.detekt.ordering

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.Rule
import org.jetbrains.kotlin.psi.KtAnonymousInitializer
import org.jetbrains.kotlin.psi.KtClassBody
import org.jetbrains.kotlin.psi.KtDeclaration
import org.jetbrains.kotlin.psi.KtFile
import org.jetbrains.kotlin.psi.KtNamedFunction
import org.jetbrains.kotlin.psi.KtObjectDeclaration
import org.jetbrains.kotlin.psi.KtProperty
import org.jetbrains.kotlin.psi.KtSecondaryConstructor

/**
 * Reports a declaration out of the order of the code-ordering rule: properties, secondary constructors,
 * functions, `init` blocks, which call the functions above them, and the companion object last. Nested classes
 * and objects other than the companion are not ordered. The order of functions among themselves is left to
 * [CalleeBeforeCaller]. At file level a preview fixture (`PREVIEW_*`, `Preview*`) is not ordered either: the
 * preview block closes a composable file, its fixtures before the previews that use them.
 */
internal class DeclarationOrder(config: Config) : Rule(config, DESCRIPTION) {

    private enum class Section(val description: String) {
        PROPERTY("properties"),
        SECONDARY_CONSTRUCTOR("secondary constructors"),
        FUNCTION("functions"),
        INIT_BLOCK("init blocks"),
        COMPANION_OBJECT("the companion object")
    }

    private fun sectionOf(declaration: KtDeclaration): Section? {
        return when {
            declaration is KtProperty -> Section.PROPERTY
            declaration is KtSecondaryConstructor -> Section.SECONDARY_CONSTRUCTOR
            declaration is KtNamedFunction -> Section.FUNCTION
            declaration is KtAnonymousInitializer -> Section.INIT_BLOCK
            declaration is KtObjectDeclaration && declaration.isCompanion() -> Section.COMPANION_OBJECT
            else -> null
        }
    }

    private fun isPreviewFixture(declaration: KtDeclaration): Boolean {
        if (declaration !is KtProperty) return false
        val name = declaration.name.orEmpty()
        return PREVIEW_PREFIXES.any { prefix -> name.startsWith(prefix) }
    }

    private fun describe(declaration: KtDeclaration, section: Section): String {
        return if (section == Section.INIT_BLOCK) "The init block" else "'${declaration.name}'"
    }

    private fun checkScope(declarations: List<KtDeclaration>) {
        var latest: Section? = null
        declarations.forEach { declaration ->
            val section = sectionOf(declaration) ?: return@forEach
            val previous = latest
            if (previous == null || section >= previous) {
                latest = section
                return@forEach
            }
            val message = "${describe(declaration, section)} goes before ${previous.description}: " +
                "properties, secondary constructors, functions, init blocks, then the companion object."
            report(Finding(Entity.from(declaration), message))
        }
    }

    override fun visitKtFile(file: KtFile) {
        super.visitKtFile(file)
        checkScope(file.declarations.filterNot { declaration -> isPreviewFixture(declaration) })
    }

    override fun visitClassBody(classBody: KtClassBody) {
        super.visitClassBody(classBody)
        checkScope(classBody.declarations)
    }

    private companion object {
        const val DESCRIPTION = "Declarations follow the code-ordering rule: properties, secondary constructors, " +
            "functions, init blocks, then the companion object."

        @JvmField
        val PREVIEW_PREFIXES = listOf("PREVIEW_", "Preview")
    }
}
