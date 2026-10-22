package ru.astrainteractive.klibs.rules.konsist.layout

/**
 * Where a package sits inside a unit: `<unit root>[/<concept>[/<sub-concept>]]/<kind>[/<sub-kind>][/<below>]`.
 *
 * @property concepts the concept packages above the kind, empty for a kind directly under the unit root
 * @property kind the kind, or `null` for the unit root and for a concept package
 * @property kindWord the segment that named the kind, `null` for an implementation sitting directly in a concept
 * package of a technology unit
 * @property subKind the segment right below Compose's `composable`, the one kind that may hold sub-packages
 * (`screen`, `component`, `layout`); `null` for any other kind
 * @property below the segments under the last kind package, or under the sub-kind, which must not exist
 */
internal data class Placement(
    val concepts: List<String>,
    val kind: Kind?,
    val kindWord: String?,
    val subKind: String?,
    val below: List<String>
) {
    fun describe(): String {
        val path = concepts.plus(listOfNotNull(kindWord, subKind)).joinToString("/")
        return if (path.isEmpty()) "the unit root" else "'$path'"
    }
}
