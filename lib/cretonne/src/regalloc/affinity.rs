//! Value affinity for register allocation.
//!
//! An SSA value's affinity is a hint used to guide the register allocator. It specifies the class
//! of allocation that is likely to cause the least amount of fixup moves in order to satisfy
//! instruction operand constraints.
//!
//! For values that want to be in registers, the affinity hint includes a register class or
//! subclass. This is just a hint, and the register allocator is allowed to pick a register from a
//! larger register class instead.

use isa::{RegInfo, RegClassIndex, OperandConstraint, ConstraintKind};

/// Preferred register allocation for an SSA value.
#[derive(Clone, Copy)]
pub enum Affinity {
    /// Don't care. This value can go anywhere.
    Any,

    /// This value should be placed in a spill slot on the stack.
    Stack,

    /// This value prefers a register from the given register class.
    Reg(RegClassIndex),
}

impl Default for Affinity {
    fn default() -> Self {
        Affinity::Any
    }
}

impl Affinity {
    /// Create an affinity that satisfies a single constraint.
    ///
    /// This will never create the indifferent `Affinity::Any`.
    /// Use the `Default` implementation for that.
    pub fn new(constraint: &OperandConstraint) -> Affinity {
        if constraint.kind == ConstraintKind::Stack {
            Affinity::Stack
        } else {
            Affinity::Reg(constraint.regclass.into())
        }
    }

    /// Merge an operand constraint into this affinity.
    ///
    /// Note that this does not guarantee that the register allocator will pick a register that
    /// satisfies the constraint.
    pub fn merge(&mut self, constraint: &OperandConstraint, reg_info: &RegInfo) {
        match *self {
            Affinity::Any => *self = Affinity::new(constraint),
            Affinity::Reg(rc) => {
                // If the preferred register class is a subclass of the constraint, there's no need
                // to change anything.
                if constraint.kind != ConstraintKind::Stack &&
                   !constraint.regclass.has_subclass(rc) {
                    // If the register classes don't overlap, `intersect` returns `None`, and we
                    // just keep our previous affinity.
                    if let Some(subclass) = constraint.regclass.intersect(reg_info.rc(rc)) {
                        // This constraint shrinks our preferred register class.
                        *self = Affinity::Reg(subclass);
                    }
                }
            }
            Affinity::Stack => {}
        }
    }
}
