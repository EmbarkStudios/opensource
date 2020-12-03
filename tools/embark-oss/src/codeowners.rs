use std::collections::HashSet;
///! Parsing and introspection of GitHub CODEOWNERS files.

#[derive(Debug, PartialEq, Eq)]
pub struct CodeOwners {
    /// CODEOWNERS files are ordered so we use a Vec of pairs rather than a
    /// a standard Rust dictionary type.
    ///
    /// Assignments later in the vector have higher precedence than those
    /// earlier.
    assignments: Vec<Assignment>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Assignment {
    /// A git compatible glob that specifies which files this assignment applies to.
    file_pattern: String,
    /// A collection of GitHub usernames or emails for the users that own this
    /// code section.
    owners: HashSet<String>,
}

impl CodeOwners {
    pub fn new(source: &str) -> Self {
        let assignments = source
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.starts_with("#"))
            .flat_map(|line| {
                let mut iter = line.split_whitespace().map(String::from);
                Some(Assignment {
                    file_pattern: iter.next()?,
                    owners: iter.collect(),
                })
            })
            .filter(|assignment| !assignment.owners.is_empty())
            .collect();
        Self { assignments }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parsing() {
        assert_eq!(
            CodeOwners::new(""),
            CodeOwners {
                assignments: vec![]
            }
        );

        assert_eq!(
            CodeOwners::new("# * lpil"),
            CodeOwners {
                assignments: vec![]
            }
        );

        assert_eq!(
            CodeOwners::new("* lpil"),
            CodeOwners {
                assignments: vec![Assignment {
                    file_pattern: "*".to_string(),
                    owners: hashset(&["lpil"])
                }]
            }
        );

        assert_eq!(
            CodeOwners::new("* lpil arirawr"),
            CodeOwners {
                assignments: vec![Assignment {
                    file_pattern: "*".to_string(),
                    owners: hashset(&["lpil", "arirawr"])
                }]
            }
        );

        assert_eq!(
            CodeOwners::new(
                "* lpil arirawr
# comment 
    # comment 
left XAMPPRocky

no-maintainers
right/ok celialewis3     soniasingla    \n"
            ),
            CodeOwners {
                assignments: vec![
                    Assignment {
                        file_pattern: "*".to_string(),
                        owners: ["lpil", "arirawr"]
                            .iter()
                            .cloned()
                            .map(String::from)
                            .collect()
                    },
                    Assignment {
                        file_pattern: "left".to_string(),
                        owners: hashset(&["XAMPPRocky"])
                    },
                    Assignment {
                        file_pattern: "right/ok".to_string(),
                        owners: hashset(&["soniasingla", "celialewis3"])
                    }
                ]
            }
        );
    }

    fn hashset(members: &[&str]) -> HashSet<String> {
        members.iter().cloned().map(String::from).collect()
    }
}
