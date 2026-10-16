package ru.astrainteractive.klibs.rules.konsist.layout

/**
 * A role package of a context, from the menu of the package-layout rule.
 */
internal enum class Role(val segment: String, val isEntryPoint: Boolean) {
    COMMAND("command", isEntryPoint = true),
    EVENT("event", isEntryPoint = true),
    ROUTE("route", isEntryPoint = true),
    COMPOSABLE("composable", isEntryPoint = true),
    VIEWMODEL("viewmodel", isEntryPoint = true),
    SERVICE("service", isEntryPoint = true),
    JOB("job", isEntryPoint = true),
    MODEL("model", isEntryPoint = false),
    DATA("data", isEntryPoint = false),
    INTEGRATION("integration", isEntryPoint = false),
    MAPPING("mapping", isEntryPoint = false),
    UTIL("util", isEntryPoint = false),
    DI("di", isEntryPoint = false);

    companion object {
        fun fromSegment(segment: String): Role? = entries.find { role -> role.segment == segment }
    }
}
