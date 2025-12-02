use std::fmt::Display;
use std::ops::{Deref, DerefMut};

use proc_macro2::Span;

#[derive(Clone, Copy)]
pub(crate) struct SpannedValue<T> {
    inner: T,
    span: Span,
}

impl<T> SpannedValue<T> {
    pub(crate) fn new(inner: T, span: Span) -> Self {
        Self { inner, span }
    }

    pub(crate) fn span(&self) -> Span {
        self.span
    }
}

impl<T> Deref for SpannedValue<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for SpannedValue<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T: Display> Display for SpannedValue<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for SpannedValue<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.inner, f)
    }
}

impl<T: PartialEq> PartialEq for SpannedValue<T> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<T: Eq> Eq for SpannedValue<T> {}

impl<T: PartialOrd> PartialOrd for SpannedValue<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.inner.partial_cmp(&other.inner)
    }
}

impl<T: Ord> Ord for SpannedValue<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.inner.cmp(other)
    }
}
