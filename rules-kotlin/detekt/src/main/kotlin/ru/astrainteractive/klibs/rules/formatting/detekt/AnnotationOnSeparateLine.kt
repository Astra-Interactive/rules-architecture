package ru.astrainteractive.klibs.rules.formatting.detekt

import com.intellij.psi.PsiComment
import com.intellij.psi.PsiWhiteSpace
import com.intellij.psi.util.PsiTreeUtil
import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.Rule
import org.jetbrains.kotlin.psi.KtAnnotationEntry
import org.jetbrains.kotlin.psi.KtDeclaration
import org.jetbrains.kotlin.psi.KtModifierList

/** Reports declaration annotations followed by another annotation or declaration on the same line. */
internal class AnnotationOnSeparateLine(config: Config) : Rule(config, DESCRIPTION) {

    override fun visitAnnotationEntry(annotationEntry: KtAnnotationEntry) {
        super.visitAnnotationEntry(annotationEntry)
        val modifiers = annotationEntry.parent as? KtModifierList ?: return
        val declaration = modifiers.parent as? KtDeclaration ?: return
        var next = PsiTreeUtil.nextLeaf(annotationEntry)
        while (next is PsiWhiteSpace || next is PsiComment) {
            next = PsiTreeUtil.nextLeaf(next)
        }
        val following = next ?: return
        if (following.textRange.startOffset >= declaration.textRange.endOffset) return
        val source = annotationEntry.containingFile.viewProvider.contents
        val gap = source.subSequence(annotationEntry.textRange.endOffset, following.textRange.startOffset)
        var previous = PsiTreeUtil.prevLeaf(annotationEntry)
        while (previous is PsiWhiteSpace || previous is PsiComment) {
            previous = PsiTreeUtil.prevLeaf(previous)
        }
        val precedingOffset = previous?.textRange?.endOffset ?: declaration.textRange.startOffset
        val prefix = source.subSequence(precedingOffset, annotationEntry.textRange.startOffset)
        val hasFollowingNewline = gap.contains('\n') || gap.contains('\r')
        val hasPrecedingNewline = prefix.contains('\n') || prefix.contains('\r')
        val hasPrecedingModifier = precedingOffset > declaration.textRange.startOffset && !hasPrecedingNewline
        if (hasFollowingNewline && !hasPrecedingModifier) return
        report(Finding(Entity.from(annotationEntry), DESCRIPTION))
    }

    private companion object {
        const val DESCRIPTION = "Place each declaration annotation on a separate line above the declaration."
    }
}
