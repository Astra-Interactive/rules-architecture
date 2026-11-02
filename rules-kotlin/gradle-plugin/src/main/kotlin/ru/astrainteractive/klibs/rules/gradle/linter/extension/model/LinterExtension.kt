package ru.astrainteractive.klibs.rules.gradle.linter.extension.model

import org.gradle.api.provider.Property
import org.gradle.api.provider.SetProperty
import ru.astrainteractive.klibs.rules.core.unit.model.ProductKind
import ru.astrainteractive.klibs.rules.core.unit.model.UnitKind
import ru.astrainteractive.klibs.rules.core.unit.model.UnitPolicy
import ru.astrainteractive.klibs.rules.core.unit.model.UnitRole
import ru.astrainteractive.klibs.rules.core.unit.util.toPolicy

public abstract class LinterExtension {
    internal var configuredUnitPolicy: UnitPolicy? = null
        private set

    internal var configuredUnitKind: UnitKind? = null
        private set

    internal var configuredProductKind: ProductKind? = null
        private set

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

    /**
     * The project's UI kinds whose parts are extension functions of the kind's context (`slot`:
     * `fun SlotContext.backSlot()` in `slot/BackSlot.kt`): there an extension is the part itself, not a `util`.
     */
    public abstract val extensionKinds: SetProperty<String>

    public fun unit(value: UnitRole) {
        val policy = value.toPolicy()
        require(configuredUnitKind == null || configuredUnitKind == value.kind) {
            "Conflicting linter unit roles: $configuredUnitKind and ${value.kind}"
        }
        require(configuredUnitPolicy == null || configuredUnitPolicy == policy) {
            "Conflicting linter.unit policies for ${value.kind}: $configuredUnitPolicy and $policy"
        }
        configuredUnitKind = value.kind
        configuredUnitPolicy = policy
    }

    public fun unitKind(value: UnitKind) {
        require(configuredUnitKind == null || configuredUnitKind == value) {
            "Conflicting linter.unitKind declarations: $configuredUnitKind and $value"
        }
        configuredUnitKind = value
    }

    public fun productKind(value: ProductKind) {
        require(configuredProductKind == null || configuredProductKind == value) {
            "Conflicting linter.productKind declarations: $configuredProductKind and $value"
        }
        configuredProductKind = value
    }
}
