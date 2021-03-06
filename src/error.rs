use eyre::Chain;
use std::{error::Error, fmt::Write};

/// Write the error and the cause
pub fn cause_string(error: &(dyn Error + 'static), should_indent: bool) -> String {
    let mut f = String::new();

    macro_rules! indent {
        () => {
            if should_indent {
                write!(f, "    ").unwrap();
            }
        };
    }

    indent!();
    writeln!(f, "{}", error).unwrap();

    if let Some(cause) = error.source() {
        indent!();
        writeln!(f, "Caused by:").unwrap();
        for (i, error) in Chain::new(cause).enumerate() {
            indent!();
            writeln!(f, "    {}: {}", i, error).unwrap();
        }
    }
    f
}
