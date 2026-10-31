package ru.astrainteractive.klibs.rules.gradle.linter.extension.model

import org.gradle.api.provider.Property
import org.gradle.api.provider.SetProperty

public abstract class LinterExtension {
    /**
     * `context`, `library`, `api`, `contract`, `entry-point` or `technology`; detected when not set.
     */
    public abstract val unitKind: Property<String>

    /**
     * Detected from the sources when not set.
     */
    public abstract val rootPackage: Property<String>

    /**
     * The project's own words for entry-point kinds (`slot`, `router`), added to the defaults of the package-layout
     * rule.
     */
    public abstract val entryPointWords: SetProperty<String>

    /**
     * The project's own technology package words (`mongo`), added to the defaults.
     */
    public abstract val technologyWords: SetProperty<String>

    /**
     * Inherently plural concepts the project uses as package names (`goods`), added to `news` and `settings`.
     */
    public abstract val pluralConcepts: SetProperty<String>

    /**
     * Supertypes of the classes the platform constructs (`WatchFaceService`), added to the defaults (`Application`,
     * `Activity`, `Service`, `Worker`, …): such a class sits in the root package of the entry-point unit.
     */
    public abstract val platformClasses: SetProperty<String>
}
