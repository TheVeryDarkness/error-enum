//! Nested `#[diag(nested)]` wrappers merge numbers and delegate the rest.
#![expect(clippy::unwrap_used)]

use error_enum::{error_type, ErrorType};

error_type! {
    #[derive(Debug)]
    Inner {
        #[diag(kind = "Error")]
        {
            #[diag(number = "23")]
            #[diag(msg = "inner failure")]
            Fail,
        }
    }
}

error_type! {
    #[derive(Debug)]
    Outer {
        #[diag(kind = "Error")]
        {
            #[diag(number = "01")]
            #[diag(nested)]
            Wrapped(Inner),
        }
    }
}

fn main() {
    let err = Outer::Wrapped(Inner::Fail);
    assert_eq!(err.number().as_ref(), "0123");
    assert_eq!(err.code().as_ref(), "E0123");
    assert_eq!(err.to_string(), "inner failure");
    println!("{err} ({})", err.code());
}
