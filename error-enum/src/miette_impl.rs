use crate::{ErrorEnum, Kind};
use miette::{Diagnostic, ReportHandler, Severity};
use std::{error::Error, fmt};

pub(crate) struct Wrapper<'a, T: ?Sized>(pub(crate) &'a T);

impl<T: ErrorEnum + 'static> Wrapper<'_, T> {
    pub(crate) fn fmt_with(&self, handler: &impl ReportHandler) -> String {
        WrapperWithHandler(self.0, handler).to_string()
    }
}

impl<T: ErrorEnum + ?Sized> fmt::Debug for Wrapper<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.primary_message())
    }
}
impl<T: ErrorEnum + ?Sized> fmt::Display for Wrapper<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.primary_message())
    }
}
impl<T: ErrorEnum + ?Sized> Error for Wrapper<'_, T> {}

impl<T: ErrorEnum + ?Sized> Diagnostic for Wrapper<'_, T> {
    fn code<'a>(&'a self) -> Option<Box<dyn fmt::Display + 'a>> {
        Some(Box::new(self.0.number()))
    }
    fn severity(&self) -> Option<Severity> {
        match self.0.kind() {
            Kind::Error => Some(Severity::Error),
            Kind::Warn => Some(Severity::Warning),
        }
    }
}

struct WrapperWithHandler<'a, T, H: ?Sized>(&'a T, &'a H);

impl<T: ErrorEnum + 'static, H: ReportHandler + ?Sized> fmt::Display
    for WrapperWithHandler<'_, T, H>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.1.display(self.0, f)
    }
}
