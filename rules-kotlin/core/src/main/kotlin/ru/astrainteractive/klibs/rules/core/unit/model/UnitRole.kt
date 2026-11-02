package ru.astrainteractive.klibs.rules.core.unit.model

/** Role configuration before it is normalized into a transportable [UnitContext]. */
public sealed class UnitRole(
    public val kind: UnitKind,
    public val checks: Checks
) {
    public class Context(checks: Checks = Checks()) : UnitRole(UnitKind.CONTEXT, checks)

    public class Api(checks: Checks = Checks()) : UnitRole(UnitKind.API, checks)

    public class Contract(checks: Checks = Checks()) : UnitRole(UnitKind.CONTRACT, checks)

    public class EntryPoint(checks: Checks = Checks()) : UnitRole(UnitKind.ENTRY_POINT, checks)

    public class Technology(checks: Checks = Checks()) : UnitRole(UnitKind.TECHNOLOGY, checks)

    public class Fake(checks: Checks = Checks()) : UnitRole(UnitKind.FAKE, checks)

    /** Matches the stable `IMPL` role tag in the public configuration API. */
    @Suppress("ImplementationName")
    public class Impl(checks: Checks = Checks()) : UnitRole(UnitKind.IMPL, checks)

    public class Platform(checks: Checks = Checks()) : UnitRole(UnitKind.PLATFORM, checks)

    public class CoreApi(checks: Checks = Checks()) : UnitRole(UnitKind.CORE_API, checks)

    public class CorePlatform(checks: Checks = Checks()) : UnitRole(UnitKind.CORE_PLATFORM, checks)

    public class CoreUi(checks: Checks = Checks()) : UnitRole(UnitKind.CORE_UI, checks)

    public class StorageContract(checks: Checks = Checks()) : UnitRole(UnitKind.STORAGE_CONTRACT, checks)

    public class ImplUi(
        public val composition: UiComposition = UiComposition.LOCAL,
        public val implementationAccess: ImplementationAccess = ImplementationAccess.SIBLINGS,
        checks: Checks = Checks()
    ) : UnitRole(UnitKind.IMPL_UI, checks)

    public class Core(checks: Checks = Checks()) : UnitRole(UnitKind.CORE, checks)
}
