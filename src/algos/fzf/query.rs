use super::Condition;

/// TODO: docs.
#[derive(Clone, Copy)]
pub struct FzfQuery<'a> {
    conditions: &'a [Condition<'a>],
}

impl core::fmt::Debug for FzfQuery<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let s = self
            .conditions
            .iter()
            .map(|condition| format!("{:?}", condition))
            .collect::<Vec<_>>()
            .join(" && ");

        f.debug_tuple("FzfQuery").field(&s).finish()
    }
}

impl<'a> FzfQuery<'a> {
    /// TODO: docs
    #[inline(always)]
    pub(super) fn conditions(&self) -> &[Condition<'a>] {
        self.conditions
    }

    /// TODO: docs
    #[inline]
    pub(super) fn is_empty(&self) -> bool {
        self.conditions.is_empty()
    }

    /// TODO: docs
    #[inline]
    pub(super) fn new(conditions: &'a [Condition<'a>]) -> Self {
        Self { conditions }
    }
}
