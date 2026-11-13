package ru.astrainteractive.klibs.rules.konsist.check.model

/**
 * A file a worked tree of the package-layout skill lists.
 *
 * @property path the path below the unit's root package (`rating/api/RatingRepository.kt`)
 * @property note the text the tree writes after the file, up to the next file (`(interface)`)
 */
internal class SkillFile(val path: String, val note: String) {
    val directory: String = path.substringBeforeLast('/', "")
    val name: String = path.substringAfterLast('/').removeSuffix(".kt")
}
