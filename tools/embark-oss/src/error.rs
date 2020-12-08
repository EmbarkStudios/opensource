use eyre::Chain;
use std::{error::Error, io::Write};

/// Write the error and the cause
pub fn write_cause(f: &mut impl Write, error: &(dyn Error + 'static), should_indent: bool) {
    macro_rules! indent {
        () => {
            if should_indent {
                write!(f, "    ").unwrap();
            }
        };
    };

    indent!();
    write!(f, "{}", error).unwrap();

    if let Some(cause) = error.source() {
        write!(f, "\n\n").unwrap();
        indent!();
        write!(f, "Caused by:\n").unwrap();
        for (i, error) in Chain::new(cause).enumerate() {
            indent!();
            write!(f, "    {}: {}\n", i, error).unwrap();
        }
    }
}
