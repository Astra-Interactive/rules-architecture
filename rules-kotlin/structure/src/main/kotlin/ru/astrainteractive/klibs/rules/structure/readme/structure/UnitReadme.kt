package ru.astrainteractive.klibs.rules.structure.readme.structure

import ru.astrainteractive.klibs.rules.structure.unit.api.StructureCheck
import ru.astrainteractive.klibs.rules.structure.unit.model.BuildUnit
import ru.astrainteractive.klibs.rules.structure.unit.model.StructureViolation
import java.io.File

/**
 * Reports a unit without a `README.md`, or with one whose text, headings aside, is not one sentence. Sentences
 * are counted by their terminal `.`, `!` or `?`, after removing the abbreviations `e.g.`, `i.e.`, `etc.` and
 * `vs.`; an `etc.` or `vs.` at the end of the text, or followed by whitespace and an uppercase letter, keeps its
 * period as a sentence stop. The periods of an initialism such as `U.S.` are not terminals, and text after the
 * last terminal counts as one more sentence. The root project is not checked: its README describes the repository.
 */
public class UnitReadme : StructureCheck {

    private fun isSentenceStop(abbreviation: String, after: String): Boolean {
        if (abbreviation.lowercase() !in CLOSING_ABBREVIATIONS) return false
        val nextWord = after.trimStart()
        val isSeparated = after.firstOrNull()?.isWhitespace() == true
        val startsSentence = nextWord.firstOrNull()?.isUpperCase() == true
        return nextWord.isEmpty() || (isSeparated && startsSentence)
    }

    private fun removeAbbreviations(text: String): String {
        return ABBREVIATION.replace(text) { match ->
            val after = text.substring(match.range.last + 1)
            if (isSentenceStop(match.groupValues[1], after)) "." else ""
        }
    }

    private fun endsInitialism(text: String, index: Int): Boolean {
        val start = text.substring(startIndex = 0, endIndex = index).indexOfLast(Char::isWhitespace) + 1
        val word = text.substring(startIndex = start, endIndex = index + 1)
        val isPairs = word.length >= MIN_INITIALISM_LENGTH && word.length % 2 == 0
        return isPairs && word.chunked(2).all { pair -> pair[0].isLetter() && pair[1] == '.' }
    }

    private fun isSentenceEnd(text: String, index: Int): Boolean {
        val character = text[index]
        if (character !in TERMINALS) return false
        val next = text.getOrNull(index + 1)
        val isFollowedByBreak = next == null || next.isWhitespace()
        return isFollowedByBreak && !(character == '.' && endsInitialism(text, index))
    }

    private fun countSentences(readme: File): Int {
        val joined = readme.readLines()
            .map(String::trim)
            .filterNot { line -> line.isEmpty() || line.startsWith("#") }
            .joinToString(separator = " ")
        val text = removeAbbreviations(joined)
        if (text.isBlank()) return 0
        val ends = text.indices.filter { index -> isSentenceEnd(text, index) }
        val tailStart = (ends.lastOrNull() ?: -1) + 1
        val hasTail = text.substring(tailStart).any { character -> !character.isWhitespace() }
        return ends.size + if (hasTail) 1 else 0
    }

    override fun check(unit: BuildUnit): List<StructureViolation> {
        if (unit.isRoot) return emptyList()
        val readme = File(unit.directory, README)
        if (!readme.isFile) {
            val message = "Add a README.md of one sentence that states the responsibility of ${unit.path}."
            return listOf(
                StructureViolation(
                    RULE,
                    README,
                    README,
                    message
                )
            )
        }
        val sentences = countSentences(readme)
        if (sentences == 1) return emptyList()
        val message = "README.md of ${unit.path} has $sentences sentences, expected one: a responsibility that " +
            "does not fit one sentence is the signal to split the unit."
        return listOf(
            StructureViolation(
                RULE,
                README,
                README,
                message
            )
        )
    }

    private companion object {
        const val RULE = "UnitReadme"
        const val README = "README.md"

        @JvmField
        val CLOSING_ABBREVIATIONS = setOf("etc", "vs")

        const val MIN_INITIALISM_LENGTH = 4

        @JvmField
        val TERMINALS = setOf(
            '.',
            '!',
            '?'
        )

        @JvmField
        val ABBREVIATION = Regex("""(?<![\p{IsAlphabetic}\p{N}_])(e\.g|i\.e|etc|vs)\.""", RegexOption.IGNORE_CASE)
    }
}
