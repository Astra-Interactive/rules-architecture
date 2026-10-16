package ru.astrainteractive.klibs.rules.detekt.kdoc

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.Rule
import org.jetbrains.kotlin.psi.KtDeclaration

/**
 * Reports Javadoc-style HTML tags (`<p>`, `<code>`, `<a href>`) in a KDoc. Code blocks and inline code are
 * ignored, so a generic type such as `List<T>` written as code does not count.
 */
internal class KDocHtmlTag(config: Config) : Rule(config, DESCRIPTION) {

    private fun findTags(text: String): List<String> {
        val prose = text.replace(FENCED_CODE, "").replace(INLINE_CODE, "")
        return HTML_TAG.findAll(prose).map { match -> match.value }.distinct().toList()
    }

    override fun visitDeclaration(dcl: KtDeclaration) {
        super.visitDeclaration(dcl)
        val docComment = dcl.docComment ?: return
        val tags = findTags(docComment.text)
        if (tags.isEmpty()) return
        val message = "Replace ${tags.joinToString()} with Markdown and [ClassName] links."
        report(Finding(Entity.from(docComment), message))
    }

    private companion object {
        @JvmField
        val FENCED_CODE = Regex("```.*?```", RegexOption.DOT_MATCHES_ALL)

        @JvmField
        val INLINE_CODE = Regex("`[^`]*`")

        @JvmField
        val HTML_TAG = Regex(
            "</?(p|br|code|pre|b|i|em|strong|tt|ul|ol|li|h[1-6]|blockquote|table|tr|td|th)\\b[^>]*>|<a\\s+href[^>]*>",
            RegexOption.IGNORE_CASE
        )
        const val DESCRIPTION = "KDoc uses Markdown, never Javadoc HTML tags."
    }
}
