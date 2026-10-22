package ru.astrainteractive.klibs.rules.konsist.layout.util

import ru.astrainteractive.klibs.rules.konsist.layout.Placement

internal fun Placement.describe(): String {
    val path = concepts.plus(listOfNotNull(kindWord, subKind)).joinToString("/")
    return if (path.isEmpty()) "the unit root" else "'$path'"
}
