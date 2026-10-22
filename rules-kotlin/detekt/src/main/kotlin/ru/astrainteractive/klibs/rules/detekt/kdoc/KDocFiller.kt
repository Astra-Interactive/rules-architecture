package ru.astrainteractive.klibs.rules.detekt.kdoc

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.Rule
import org.jetbrains.kotlin.kdoc.psi.api.KDoc
import org.jetbrains.kotlin.kdoc.psi.impl.KDocTag
import org.jetbrains.kotlin.psi.KtFile
import org.jetbrains.kotlin.psi.psiUtil.collectDescendantsOfType

/**
 * Reports KDoc that says nothing the signature does not: one of the [FILLER_PHRASES], a `@return` that
 * names no edge case, a `@param` that only repeats its name, and a summary of more than four sentences before
 * the first tag.
 */
internal class KDocFiller(config: Config) : Rule(config, DESCRIPTION) {

    private fun normalize(text: String): String {
        return text.lowercase()
            .replace(WHITESPACE, " ")
            .trim()
            .removeSuffix(".")
            .removePrefix("the ")
            .removePrefix("a ")
            .removePrefix("an ")
    }

    private fun findReturnMessages(kdoc: KDoc): List<String> {
        return kdoc.getDefaultSection().findTagsByName(RETURN_TAG)
            .filter { tag -> normalize(tag.getContent()) in EMPTY_RETURNS }
            .map { "Delete the @return that restates the type, or say what is returned in the edge cases." }
    }

    private fun repeatsName(tag: KDocTag, name: String): Boolean {
        val words = name.split(WORD_START).joinToString(" ") { word -> word.lowercase() }
        val content = normalize(tag.getContent())
        return content.isEmpty() || content == words
    }

    private fun findParamMessages(kdoc: KDoc): List<String> {
        return kdoc.getDefaultSection().findTagsByName(PARAM_TAG).mapNotNull { tag ->
            val name = tag.getSubjectName() ?: return@mapNotNull null
            if (!repeatsName(tag, name)) return@mapNotNull null
            "Delete '@param $name' that repeats the name, or describe its meaning and constraints."
        }
    }

    private fun findSummaryMessage(kdoc: KDoc): String? {
        val summary = kdoc.getDefaultSection().getContent()
        val sentences = SENTENCE_END.findAll(summary).count()
        if (sentences <= MAX_SENTENCES) return null
        return "Shorten the summary to at most $MAX_SENTENCES sentences: a longer one is a sign the declaration does " +
            "too much."
    }

    private fun findMessages(kdoc: KDoc): List<String> {
        val text = kdoc.text.replace(WHITESPACE, " ")
        val fillers = FILLER_PHRASES
            .filter { phrase -> text.contains(phrase, ignoreCase = true) }
            .map { phrase -> "Delete the filler '$phrase': say what the signature cannot, or write no KDoc." }
        return fillers + findReturnMessages(kdoc) + findParamMessages(kdoc) + listOfNotNull(findSummaryMessage(kdoc))
    }

    override fun visitKtFile(file: KtFile) {
        super.visitKtFile(file)
        file.collectDescendantsOfType<KDoc>().forEach { kdoc ->
            findMessages(kdoc).forEach { message -> report(Finding(Entity.from(kdoc), message)) }
        }
    }

    private companion object {
        const val RETURN_TAG = "return"
        const val PARAM_TAG = "param"
        const val MAX_SENTENCES = 4

        @JvmField
        val WHITESPACE = Regex("""\s+""")

        @JvmField
        val WORD_START = Regex("(?=[A-Z])|_")

        @JvmField
        val SENTENCE_END = Regex("""[.!?](\s|$)""")

        @JvmField
        val EMPTY_RETURNS = setOf(
            "",
            "result",
            "returned value",
            "value"
        )

        @JvmField
        val FILLER_PHRASES = listOf(
            "This method is responsible for",
            "This function allows",
            "This class is responsible",
            "Returns the result of the operation",
            "is used to",
            "is intended for"
        )
        const val DESCRIPTION = "KDoc says only what the signature does not."
    }
}
