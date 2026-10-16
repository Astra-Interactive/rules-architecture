package ru.astrainteractive.klibs.rules.structure.readme

import ru.astrainteractive.klibs.rules.structure.unit.BuildUnit
import ru.astrainteractive.klibs.rules.structure.unit.StructureCheck
import ru.astrainteractive.klibs.rules.structure.unit.StructureViolation
import java.io.File

/**
 * Reports a unit without a `README.md`, or with one whose text, headings aside, is not one sentence. Sentences
 * are counted by their terminal `.`, `!` or `?`, after removing the abbreviations `e.g.`, `i.e.`, `etc.` and
 * `vs.`. The root project is not checked: its README describes the repository.
 */
public class UnitReadme : StructureCheck {

    private fun countSentences(readme: File): Int {
        val text = readme.readLines()
            .map { line -> line.trim() }
            .filterNot { line -> line.isEmpty() || line.startsWith("#") }
            .joinToString(separator = " ")
            .replace(ABBREVIATION, "")
        if (text.isBlank()) return 0
        return SENTENCE_END.findAll(text).count().coerceAtLeast(1)
    }

    override fun check(unit: BuildUnit): List<StructureViolation> {
        if (unit.isRoot) return emptyList()
        val readme = File(unit.directory, README)
        if (!readme.isFile) {
            val message = "Add a README.md of one sentence that states the responsibility of ${unit.path}."
            return listOf(StructureViolation(RULE, README, README, message))
        }
        val sentences = countSentences(readme)
        if (sentences == 1) return emptyList()
        val message = "README.md of ${unit.path} has $sentences sentences, expected one: a responsibility that " +
            "does not fit one sentence is the signal to split the unit."
        return listOf(StructureViolation(RULE, README, README, message))
    }

    private companion object {
        const val RULE = "UnitReadme"
        const val README = "README.md"

        @JvmField
        val SENTENCE_END = Regex("""[.!?](\s|$)""")

        @JvmField
        val ABBREVIATION = Regex("""\b(e\.g|i\.e|etc|vs)\.""", RegexOption.IGNORE_CASE)
    }
}
