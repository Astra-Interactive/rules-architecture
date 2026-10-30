package ru.astrainteractive.klibs.rules.konsist.layout.util

import ru.astrainteractive.klibs.rules.konsist.layout.model.Kind
import ru.astrainteractive.klibs.rules.konsist.layout.model.Vocabulary

/**
 * The kind a package segment names, from the project's own words first, then from the defaults of [Kind].
 */
internal fun Vocabulary.kindOf(segment: String): Kind? {
    return when (segment) {
        in entryPointWords -> Kind.ENTRY_POINT
        in technologyWords -> Kind.IMPLEMENTATION
        else -> Kind.entries.find { kind -> segment in kind.words }
    }
}
