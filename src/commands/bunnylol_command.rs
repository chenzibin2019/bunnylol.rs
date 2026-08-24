/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use serde::Serialize;

/// Information about a fixed option accepted by a Bunnylol command.
///
/// `values` contains equivalent spellings for the same option. When
/// `requires_argument` is true, clients should keep accepting input after the
/// selected option instead of executing it immediately.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct BunnylolCommandOption {
    pub values: Vec<String>,
    pub description: String,
    pub requires_argument: bool,
}

impl BunnylolCommandOption {
    pub fn new(values: &[&str], description: &str) -> Self {
        Self {
            values: values.iter().map(|s| s.to_string()).collect(),
            description: description.to_string(),
            requires_argument: false,
        }
    }

    pub fn requiring_argument(mut self) -> Self {
        self.requires_argument = true;
        self
    }
}

/// Information about a registered command binding
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct BunnylolCommandInfo {
    pub bindings: Vec<String>,
    pub description: String,
    pub example: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub options: Vec<BunnylolCommandOption>,
}

impl BunnylolCommandInfo {
    // Create a new BunnylolCommandInfo
    pub fn new(bindings: &[&str], description: &str, example: &str) -> Self {
        BunnylolCommandInfo {
            bindings: bindings.iter().map(|s| s.to_string()).collect(),
            description: description.to_string(),
            example: example.to_string(),
            options: Vec::new(),
        }
    }

    /// Attach fixed command options for discovery clients such as Raycast.
    pub fn with_options(mut self, options: Vec<BunnylolCommandOption>) -> Self {
        self.options = options;
        self
    }
}

/// Bunnylol Command trait that all URL builders must implement
pub trait BunnylolCommand {
    /// All command strings that trigger this binding (e.g., ["gh", "github"])
    const BINDINGS: &'static [&'static str];

    /// Process the command arguments and return the appropriate URL
    fn process_args(args: &str) -> String;

    /// Get the command portion from the full arguments string
    fn get_command_args(args: &str) -> &str {
        // Check if args starts with any of the bindings
        for binding in Self::BINDINGS {
            if args.split_whitespace().next() == Some(*binding) {
                if args.len() <= binding.len() {
                    return "";
                } else {
                    return args[binding.len()..].trim_start();
                }
            }
        }
        args
    }

    /// Check if this binding matches the given command
    fn matches_command(command: &str) -> bool {
        Self::BINDINGS.contains(&command)
    }

    /// Get information about this command (description and examples)
    fn get_info() -> BunnylolCommandInfo;
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock command for testing
    struct TestCommand;

    impl BunnylolCommand for TestCommand {
        const BINDINGS: &'static [&'static str] = &["test", "t"];

        fn process_args(args: &str) -> String {
            let query = Self::get_command_args(args);
            if query.is_empty() {
                "https://test.com".to_string()
            } else {
                format!("https://test.com/search?q={}", query)
            }
        }

        fn get_info() -> BunnylolCommandInfo {
            BunnylolCommandInfo::new(Self::BINDINGS, "Test command", "test query")
        }
    }

    #[test]
    fn test_bunnylol_command_get_command_args() {
        assert_eq!(TestCommand::get_command_args("test"), "");
        assert_eq!(TestCommand::get_command_args("test hello"), "hello");
        assert_eq!(
            TestCommand::get_command_args("test hello world"),
            "hello world"
        );
    }

    #[test]
    fn test_bunnylol_command_matches_command() {
        assert!(TestCommand::matches_command("test"));
        assert!(TestCommand::matches_command("t"));
        assert!(!TestCommand::matches_command("other"));
    }

    #[test]
    fn test_bunnylol_command_process_args() {
        assert_eq!(TestCommand::process_args("test"), "https://test.com");
        assert_eq!(TestCommand::process_args("t"), "https://test.com");
        assert_eq!(
            TestCommand::process_args("test hello"),
            "https://test.com/search?q=hello"
        );
        assert_eq!(
            TestCommand::process_args("t hello"),
            "https://test.com/search?q=hello"
        );
    }

    #[test]
    fn test_command_info_options() {
        let info = BunnylolCommandInfo::new(&["test"], "Test command", "test")
            .with_options(vec![
                BunnylolCommandOption::new(&["settings"], "Open settings"),
                BunnylolCommandOption::new(&["provider"], "Choose provider")
                    .requiring_argument(),
            ]);

        assert_eq!(info.options.len(), 2);
        assert_eq!(info.options[0].values, vec!["settings"]);
        assert!(!info.options[0].requires_argument);
        assert!(info.options[1].requires_argument);
    }
}
