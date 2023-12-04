use std::fmt::Display;

pub trait ErrPrint {
    #[must_use]
    fn err_print(self, msg: String) -> Self;
}

impl<T, E: Display> ErrPrint for Result<T, E> {
    fn err_print(self, msg: String) -> Self {
        if let Err(err) = &self {
            eprintln!("{msg}. Cause: {err}");
        }
        self
    }
}

impl<T> ErrPrint for Option<T> {
    fn err_print(self, msg: String) -> Self {
        if self.is_none() {
            eprintln!("{msg}. Cause the value is none.");
        }
        self
    }
}
