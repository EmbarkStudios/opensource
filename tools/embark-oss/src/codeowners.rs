use eyre::{eyre, WrapErr};
use std::collections::HashSet;

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

impl Assignment {
    pub fn from_line(line: &str) -> eyre::Result<Self> {
        let mut iter = line.split_whitespace().map(String::from);
        let file_pattern = iter
            .next()
            .ok_or_else(|| eyre!("No file pattern for code owners line"))?;
        let owners = iter
            .map(validate_name_format)
            .collect::<eyre::Result<HashSet<String>>>()
            .wrap_err_with(|| format!("Unable to parse code owners for {}", file_pattern))?;
        if owners.is_empty() {
            return Err(eyre!("File pattern `{}` has no owners", file_pattern));
        }
        Ok(Assignment {
            file_pattern,
            owners,
        })
    }
}

fn validate_name_format(name: String) -> eyre::Result<String> {
    if name.starts_with("@") {
        Ok(name.trim_start_matches("@").to_string())
    } else {
        Err(eyre!("Code owner `{}` does not start with an @", name))
    }
}

impl CodeOwners {
    pub fn new(source: &str) -> eyre::Result<Self> {
        let assignments = source
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.starts_with("#"))
            .filter(|line| !line.is_empty())
            .map(Assignment::from_line)
            .collect::<eyre::Result<_>>()?;
        Ok(Self { assignments })
    }

    pub fn primary_maintainers(&self) -> Option<&HashSet<String>> {
        self.assignments
            .iter()
            .find(|assignment| assignment.file_pattern == "*")
            .map(|assignment| &assignment.owners)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parsing() {
        assert_eq!(
            CodeOwners::new("").unwrap(),
            CodeOwners {
                assignments: vec![]
            }
        );

        assert_eq!(
            CodeOwners::new("# * @lpil").unwrap(),
            CodeOwners {
                assignments: vec![]
            }
        );

        assert_eq!(
            CodeOwners::new("* @lpil").unwrap(),
            CodeOwners {
                assignments: vec![Assignment {
                    file_pattern: "*".to_string(),
                    owners: hashset(&["lpil"])
                }]
            }
        );

        assert_eq!(
            CodeOwners::new("* @lpil @arirawr").unwrap(),
            CodeOwners {
                assignments: vec![Assignment {
                    file_pattern: "*".to_string(),
                    owners: hashset(&["lpil", "arirawr"])
                }]
            }
        );

        assert_eq!(
            CodeOwners::new("* @lpil arirawr").unwrap_err().to_string(),
            "Unable to parse code owners for *",
        );

        assert_eq!(
            CodeOwners::new(
                "* @lpil @arirawr
# comment 
    # comment 
left @XAMPPRocky

right/ok @celialewis3     @soniasingla    \n"
            )
            .unwrap(),
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
