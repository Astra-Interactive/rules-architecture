package ru.astrainteractive.klibs.rules.gradle.linter.extension

import org.gradle.api.provider.Property

public abstract class LinterExtension {
    /**
     * `context`, `library`, `api`, `contract`, `entry-point` or `technology`; detected when not set.
     */
    public abstract val unitKind: Property<String>

    /**
     * Detected from the sources when not set.
     */
    public abstract val rootPackage: Property<String>
}
