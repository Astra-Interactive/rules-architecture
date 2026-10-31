package ru.astrainteractive.klibs.rules.konsist.check.placement.konsist

import com.lemonappdev.konsist.api.declaration.KoFileDeclaration
import ru.astrainteractive.klibs.rules.konsist.check.konsist.ArchitectureCheck
import ru.astrainteractive.klibs.rules.konsist.check.konsist.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.check.model.util.baseName
import ru.astrainteractive.klibs.rules.konsist.check.model.util.topLevelClasses
import ru.astrainteractive.klibs.rules.konsist.layout.model.Kind
import ru.astrainteractive.klibs.rules.konsist.layout.model.UnitKind

/**
 * Reports a main source file of an entry-point unit that is neither at the unit root nor in `di`: the entry point
 * holds the classes the platform constructs, in its root package, and `RootModule`, nothing else. A class whose
 * direct supertype is one the platform constructs (`Application`, `Service`, `CoroutineWorker`, the project's own
 * `platformClasses`) is sent to the root package; any other file out of the unit.
 */
internal class EntryPointShape : ArchitectureCheck {
    override val unitKinds: Set<UnitKind> = setOf(UnitKind.ENTRY_POINT)

    private fun isPlatformClassFile(sources: UnitSources, file: KoFileDeclaration): Boolean {
        val platformClasses = PLATFORM_CLASSES + sources.layout.vocabulary.platformClasses
        return file.topLevelClasses().any { declaration ->
            declaration.parents().any { parent ->
                parent.name.substringBefore('<').substringAfterLast('.') in platformClasses
            }
        }
    }

    override fun check(sources: UnitSources): List<Violation> {
        return sources.mainFiles
            .filter { file ->
                val placement = sources.placementOf(file) ?: return@filter true
                val isRoot = placement.concepts.isEmpty() && placement.kind == null
                !isRoot && placement.kind != Kind.DI
            }
            .map { file ->
                val message = if (isPlatformClassFile(sources, file)) {
                    "Move ${file.baseName} to the root package of the entry point: the platform constructs it, " +
                        "and the entry point keeps such classes beside di."
                } else {
                    "Move ${file.baseName} out of the entry point: the entry point holds the classes the platform " +
                        "constructs and RootModule, nothing else; the code goes to the unit of its sub-context or " +
                        "to core."
                }
                sources.createViolation(
                    RULE,
                    file,
                    file.baseName,
                    message
                )
            }
    }

    private companion object {
        const val RULE = "EntryPointShape"

        @JvmField
        val PLATFORM_CLASSES = setOf(
            "Activity",
            "AppCompatActivity",
            "Application",
            "BroadcastReceiver",
            "ComponentActivity",
            "ContentProvider",
            "CoroutineWorker",
            "FragmentActivity",
            "JavaPlugin",
            "LifecycleService",
            "ListenableWorker",
            "Service",
            "TileService",
            "Worker"
        )
    }
}
