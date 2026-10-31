package ru.astrainteractive.klibs.rules.konsist.check.declaration.konsist

import com.lemonappdev.konsist.api.declaration.KoBaseDeclaration
import com.lemonappdev.konsist.api.provider.KoKDocProvider
import com.lemonappdev.konsist.api.provider.KoNameProvider
import com.lemonappdev.konsist.api.provider.KoTextProvider
import ru.astrainteractive.klibs.rules.konsist.check.konsist.ArchitectureCheck
import ru.astrainteractive.klibs.rules.konsist.check.konsist.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.layout.model.UnitKind

/**
 * Reports KDoc in application code, any unit but a published library: it is written only for the failure case of
 * a contract (`@return failure …`, `@return null when …`, `@throws`) or for a reason that points outside the code,
 * recognised by a link, an issue key (`KT-12345`, `#42`) or the words bug, workaround, quirk and issue. A KDoc is
 * reported once, on the declaration it stands before: Konsist hands a class's KDoc to the properties of its primary
 * constructor too.
 */
internal class ApplicationKDoc : ArchitectureCheck {
    override val unitKinds: Set<UnitKind> = UnitKind.entries.toSet() - UnitKind.LIBRARY

    private fun isAllowed(text: String): Boolean = ALLOWED.containsMatchIn(text)

    private fun hasOwnKDoc(declaration: KoBaseDeclaration): Boolean {
        return (declaration as? KoTextProvider)?.text.orEmpty().trimStart().startsWith(KDOC_START)
    }

    override fun check(sources: UnitSources): List<Violation> {
        return sources.mainFiles.flatMap { file ->
            file.declarations(includeNested = true, includeLocal = false)
                .filter { declaration ->
                    val kDoc = (declaration as? KoKDocProvider)?.kDoc ?: return@filter false
                    hasOwnKDoc(declaration as KoBaseDeclaration) && !isAllowed(kDoc.text)
                }
                .map { declaration ->
                    val name = (declaration as? KoNameProvider)?.name.orEmpty()
                    val message = "Delete the KDoc of $name: application code carries KDoc only for the failure " +
                        "case of a contract or a reason outside the code, such as a platform quirk or a library bug."
                    sources.createViolation(
                        RULE,
                        file,
                        declaration as KoBaseDeclaration,
                        name,
                        message
                    )
                }
        }
    }

    private companion object {
        const val RULE = "ApplicationKDoc"
        const val KDOC_START = "/**"

        @JvmField
        val ALLOWED = Regex(
            """@return\s+(failure|null)\b|@throws|https?://|\b[A-Z][A-Z0-9]+-\d+\b|#\d+|\b(bug|workaround|quirk|issue)""",
            RegexOption.IGNORE_CASE
        )
    }
}
