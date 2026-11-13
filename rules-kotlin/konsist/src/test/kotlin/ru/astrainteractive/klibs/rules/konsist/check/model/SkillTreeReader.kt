package ru.astrainteractive.klibs.rules.konsist.check.model

import ru.astrainteractive.klibs.rules.core.unit.model.UnitKind

/**
 * Reads the units of a worked tree of the package-layout skill: a line that starts at the margin names a unit,
 * followed by its root package (`…data.dao`, after `com.example.<project>`) when the tree gives one and then by
 * files; an indented line lists more files of the unit above. A later block that names a unit again adds files to
 * it. A note is any text that is not a file path.
 */
internal class SkillTreeReader(
    private val project: String,
    private val unitKinds: Map<String, UnitKind>
) {
    private fun expandBraces(path: String): List<String> {
        val names = path.substringAfter('{', "").substringBefore('}', "")
        if (names.isEmpty()) return listOf(path)
        return names.split(", ").map { name -> path.substringBefore('{') + name + path.substringAfter('}') }
    }

    private fun readFiles(text: String): List<SkillFile> {
        val matches = FILE_PATH.findAll(text).toList()
        return matches.flatMapIndexed { index, match ->
            val end = matches.getOrNull(index + 1)?.range?.first ?: text.length
            val note = text.substring(match.range.last + 1, end).trim()
            expandBraces(match.value).map { path -> SkillFile(path, note) }
        }
    }

    fun read(blocks: List<String>): List<SkillUnit> {
        val roots = linkedMapOf<String, String>()
        val files = linkedMapOf<String, MutableList<SkillFile>>()
        var unitPath = ""
        blocks.flatMap(String::lines).filter(String::isNotBlank).forEach { line ->
            var listed = line.trim()
            if (!line.startsWith(' ')) {
                unitPath = line.substringBefore(' ')
                listed = line.substringAfter(' ', "").trim()
                val hasRoot = listed.startsWith(ROOT_MARK)
                val root = if (hasRoot) {
                    listed.drop(1).substringBefore(' ').removeSuffix(":")
                } else {
                    unitPath.substringAfterLast('/')
                }
                if (hasRoot) listed = listed.substringAfter(' ', "")
                roots.putIfAbsent(unitPath, "com.example.$project.$root")
            }
            files.getOrPut(unitPath, ::mutableListOf).addAll(readFiles(listed))
        }
        return roots.map { (path, root) ->
            SkillUnit(
                path = path,
                kind = unitKinds.getValue(path),
                rootPackage = root,
                files = files[path].orEmpty()
            )
        }
    }

    private companion object {
        const val ROOT_MARK = '…'

        @JvmField
        val FILE_PATH = Regex("""(?:[a-z][\w-]*/)*(?:\{[A-Za-z, ]+\}|[A-Z]\w*)\.kt""")
    }
}
