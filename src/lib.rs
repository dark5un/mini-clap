/// A simple CLI argument parser — mini-clap.
///
/// Allows defining commands with positional args, flags, named options,
/// subcommands, and automatic `--help` generation.

use std::collections::HashSet;

// ---- Core types ----

/// A command-line app definition.
#[derive(Clone)]
pub struct Command {
    name: String,
    about: String,
    args: Vec<Arg>,
    flags: Vec<Flag>,
    options: Vec<Opt>,
    subcommands: Vec<Command>,
}

impl Command {
    /// Create a new command with the given name.
    pub fn new(name: &str) -> Self {
        Command {
            name: name.to_string(),
            about: String::new(),
            args: vec![],
            flags: vec![],
            options: vec![],
            subcommands: vec![],
        }
    }

    /// Set the description of this command.
    pub fn about(mut self, about: &str) -> Self {
        self.about = about.to_string();
        self
    }

    /// Add a positional argument to this command.
    pub fn arg(mut self, arg: Arg) -> Self {
        self.args.push(arg);
        self
    }

    /// Add a boolean flag (e.g. `--verbose` or `-v`).
    pub fn flag(mut self, flag: Flag) -> Self {
        self.flags.push(flag);
        self
    }

    /// Add a named option (e.g. `--name <val>`).
    pub fn option(mut self, opt: Opt) -> Self {
        self.options.push(opt);
        self
    }

    /// Add a subcommand.
    pub fn subcommand(mut self, cmd: Command) -> Self {
        self.subcommands.push(cmd);
        self
    }

    /// Render the help text for this command.
    pub fn render_help(&self) -> String {
        let mut out = format!("Usage: {}", self.name);
        if !self.args.is_empty() {
            for a in &self.args {
                out.push_str(&format!(" <{}>", a.name));
            }
        }
        if !self.flags.is_empty() || !self.options.is_empty() {
            out.push_str(" [OPTIONS]");
        }
        if !self.subcommands.is_empty() {
            out.push_str(" [COMMAND]");
        }
        out.push('\n');

        if !self.about.is_empty() {
            out.push_str(&format!("\n{}\n", self.about));
        }

        if !self.args.is_empty() {
            out.push_str("\nArguments:\n");
            for a in &self.args {
                out.push_str(&format!("  <{}>\t{}\n", a.name, a.help));
            }
        }

        if !self.flags.is_empty() {
            out.push_str("\nFlags:\n");
            for f in &self.flags {
                let short = f.short.map(|c| format!("-{}, ", c)).unwrap_or_default();
                out.push_str(&format!("  {}{}\t{}\n", short, f.long_name(), f.help));
            }
        }

        if !self.options.is_empty() {
            out.push_str("\nOptions:\n");
            for o in &self.options {
                let short = o.short.map(|c| format!("-{}, ", c)).unwrap_or_default();
                out.push_str(&format!("  {}{}\t{}\n", short, o.long_name(), o.help));
            }
        }

        if !self.subcommands.is_empty() {
            out.push_str("\nSubcommands:\n");
            for s in &self.subcommands {
                out.push_str(&format!("  {}\t{}\n", s.name, s.about));
            }
        }

        out
    }

    /// Parse the given argument list against this command's definition.
    pub fn parse(self, args: &[String]) -> Result<ArgMatches, Error> {
        let mut positional = Vec::new();
        let mut flags_found = HashSet::new();
        let mut option_values = Vec::new();
        let mut subcommand_matches: Option<(String, Box<ArgMatches>)> = None;
        let mut i = 0;
        let known_flags: HashSet<String> = self.flags.iter().map(|f| f.name.clone()).collect();
        let known_options: HashSet<String> = self.options.iter().map(|o| o.name.clone()).collect();
        let known_subcommands: HashSet<String> =
            self.subcommands.iter().map(|s| s.name.clone()).collect();

        while i < args.len() {
            let arg = &args[i];

            // Check for subcommand match (first positional that matches)
            if !arg.starts_with('-') {
                if subcommand_matches.is_none() && known_subcommands.contains(arg.as_str()) {
                    if let Some(cmd) = self.subcommands.iter().find(|c| c.name == *arg).cloned() {
                        let rest = args[i + 1..].to_vec();
                        subcommand_matches = Some((cmd.name.clone(), Box::new(cmd.parse(&rest)?)));
                        break;
                    }
                }
                positional.push(arg.clone());
                i += 1;
                continue;
            }

            // Flag or option
            if arg.starts_with("--") {
                let name = arg.trim_start_matches("--").to_string();
                if let Some(opt) = self.options.iter().find(|o| o.name == name) {
                    // Named option with value
                    i += 1;
                    if i >= args.len() {
                        return Err(Error {
                            message: format!("option `{name}` requires a value"),
                        });
                    }
                    option_values.push(OptValue { name, value: args[i].clone() });
                } else if known_flags.contains(&name) {
                    flags_found.insert(name);
                } else {
                    return Err(Error {
                        message: format!("unknown argument `{arg}`"),
                    });
                }
            } else if arg.starts_with('-') && arg.len() == 2 {
                let c = arg.chars().nth(1).unwrap();
                // Short flag
                if let Some(flag) = self.flags.iter().find(|f| f.short == Some(c)) {
                    flags_found.insert(flag.name.clone());
                } else if let Some(opt) = self.options.iter().find(|o| o.short == Some(c)) {
                    // Short option with value
                    i += 1;
                    if i >= args.len() {
                        return Err(Error {
                            message: format!("option `-{c}` requires a value"),
                        });
                    }
                    option_values.push(OptValue { name: opt.name.clone(), value: args[i].clone() });
                } else {
                    return Err(Error {
                        message: format!("unknown argument `{arg}`"),
                    });
                }
            }
            i += 1;
        }

        // Check required options
        for opt in &self.options {
            if opt.required {
                let found = option_values.iter().any(|v| v.name == opt.name);
                if !found {
                    return Err(Error {
                        message: format!("required option `--{}` was not provided", opt.name),
                    });
                }
            }
        }

        let arg_names = self.args.iter().map(|a| a.name.clone()).collect();

        Ok(ArgMatches {
            command_name: self.name,
            arg_names,
            values: positional,
            flags_found,
            option_values,
            subcommand: subcommand_matches,
        })
    }
}

/// A positional argument definition.
#[derive(Clone)]
pub struct Arg {
    /// Name of the argument (used for display and retrieval).
    pub name: String,
    /// Help text displayed in usage.
    pub help: String,
}

impl Arg {
    /// Create a new positional argument with the given name.
    pub fn new(name: &str) -> Self {
        Arg {
            name: name.to_string(),
            help: String::new(),
        }
    }

    /// Set the help text for this argument.
    pub fn help(mut self, help: &str) -> Self {
        self.help = help.to_string();
        self
    }
}

/// A boolean flag definition (e.g. `--verbose` or `-v`).
#[derive(Clone)]
pub struct Flag {
    name: String,
    pub short: Option<char>,
    pub help: String,
}

impl Flag {
    /// Create a new flag with the given long name.
    pub fn new(name: &str) -> Self {
        Flag {
            name: name.to_string(),
            short: None,
            help: String::new(),
        }
    }

    /// Set a short alias (e.g. 'v' for `-v`).
    pub fn short(mut self, c: char) -> Self {
        self.short = Some(c);
        self
    }

    /// Set the help text for this flag.
    pub fn help(mut self, help: &str) -> Self {
        self.help = help.to_string();
        self
    }

    /// The long flag name (e.g. "--verbose").
    pub fn long_name(&self) -> String {
        format!("--{}", self.name)
    }
}

/// A named option definition with a value (e.g. `--name <val>`).
#[derive(Clone)]
pub struct Opt {
    name: String,
    pub short: Option<char>,
    pub help: String,
    pub required: bool,
}

impl Opt {
    /// Create a new option with the given name.
    pub fn new(name: &str) -> Self {
        Opt {
            name: name.to_string(),
            short: None,
            help: String::new(),
            required: false,
        }
    }

    /// Set a short alias (e.g. 'n' for `-n`).
    pub fn short(mut self, c: char) -> Self {
        self.short = Some(c);
        self
    }

    /// Set whether this option is required.
    pub fn required(mut self, val: bool) -> Self {
        self.required = val;
        self
    }

    /// The long option name (e.g. "--name").
    pub fn long_name(&self) -> String {
        format!("--{} <{}>", self.name, self.name)
    }
}

#[derive(Debug)]
struct OptValue {
    name: String,
    value: String,
}

/// The result of a successful parse.
#[derive(Debug)]
pub struct ArgMatches {
    /// The name of the matched command.
    pub command_name: String,
    /// Argument names in positional order.
    arg_names: Vec<String>,
    /// Raw positional argument values (no flags/options/subcommands).
    pub values: Vec<String>,
    /// Set of flag names that were found during parsing.
    flags_found: HashSet<String>,
    /// Values for named options.
    option_values: Vec<OptValue>,
    /// Matched subcommand, if any.
    subcommand: Option<(String, Box<ArgMatches>)>,
}

impl ArgMatches {
    /// Get a positional argument's value by name.
    pub fn get(&self, name: &str) -> Option<&str> {
        // First check option values
        if let Some(v) = self.option_values.iter().find(|o| o.name == name) {
            return Some(&v.value);
        }
        // Then check positional args
        self.arg_names
            .iter()
            .position(|n| n == name)
            .and_then(|i| self.values.get(i))
            .map(|s| s.as_str())
    }

    /// Check if a boolean flag was present.
    pub fn get_flag(&self, name: &str) -> bool {
        self.flags_found.contains(name)
    }

    /// Get the matched subcommand, if any.
    pub fn subcommand(&self) -> Option<(&str, &ArgMatches)> {
        self.subcommand
            .as_ref()
            .map(|(name, matches)| (name.as_str(), matches.as_ref()))
    }
}

/// A parse error.
#[derive(Debug)]
pub struct Error {
    /// Human-readable error description.
    pub message: String,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for Error {}

// ---- Tests ----

#[cfg(test)]
mod tests {
    use super::*;

    // Cycle 1: Define a command, parse no args → succeeds
    #[test]
    fn test_parse_empty_args_returns_ok() {
        let cmd = Command::new("myapp");
        assert!(cmd.parse(&[]).is_ok());
    }

    // Cycle 2: Positional arg — define one, parse, retrieve
    #[test]
    fn test_positional_arg_value() {
        let cmd = Command::new("myapp")
            .arg(Arg::new("input"));
        let matches = cmd.parse(&["file.txt".to_string()]).unwrap();
        assert_eq!(matches.get("input"), Some("file.txt"));
    }

    // Cycle 3: Multiple positional args — define two, retrieve by name
    #[test]
    fn test_multiple_positional_args() {
        let cmd = Command::new("myapp")
            .arg(Arg::new("source"))
            .arg(Arg::new("dest"));
        let matches = cmd.parse(&["/src".to_string(), "/dst".to_string()]).unwrap();
        assert_eq!(matches.get("source"), Some("/src"));
        assert_eq!(matches.get("dest"), Some("/dst"));
    }

    // Cycle 4: Flag — define --verbose, parse it, check presence
    #[test]
    fn test_flag_present() {
        let cmd = Command::new("myapp")
            .flag(Flag::new("verbose"));
        let matches = cmd.parse(&["--verbose".to_string()]).unwrap();
        assert!(matches.get_flag("verbose"));
    }

    #[test]
    fn test_flag_absent() {
        let cmd = Command::new("myapp")
            .flag(Flag::new("verbose"));
        let matches = cmd.parse(&[]).unwrap();
        assert!(!matches.get_flag("verbose"));
    }

    // Cycle 5: Flag with short alias (-v)
    #[test]
    fn test_flag_short_alias() {
        let cmd = Command::new("myapp")
            .flag(Flag::new("verbose").short('v'));
        let matches = cmd.parse(&["-v".to_string()]).unwrap();
        assert!(matches.get_flag("verbose"));
    }

    // Cycle 6: Named option (--name <val>)
    #[test]
    fn test_named_option_value() {
        let cmd = Command::new("myapp")
            .option(Opt::new("name"));
        let matches = cmd.parse(&["--name".to_string(), "Panos".to_string()]).unwrap();
        assert_eq!(matches.get("name"), Some("Panos"));
    }

    // Cycle 7: Required option → error on missing
    #[test]
    fn test_required_option_missing_errors() {
        let cmd = Command::new("myapp")
            .option(Opt::new("output").required(true));
        let err = cmd.parse(&[]).unwrap_err();
        assert!(err.message.contains("output"));
    }

    // Cycle 8: --help generation
    #[test]
    fn test_help_contains_name_and_args() {
        let cmd = Command::new("myapp")
            .about("A test app")
            .arg(Arg::new("input").help("The input file"))
            .flag(Flag::new("verbose").help("Show debug output"));
        let help = cmd.render_help();
        assert!(help.contains("myapp"));
        assert!(help.contains("A test app"));
        assert!(help.contains("--verbose"));
    }

    // Cycle 9: Error on unknown args
    #[test]
    fn test_unknown_flag_errors() {
        let cmd = Command::new("myapp");
        let err = cmd.parse(&["--bogus".to_string()]).unwrap_err();
        assert!(err.message.contains("bogus"));
    }

    // Cycle 10: Subcommand matching
    #[test]
    fn test_subcommand_matching() {
        let init = Command::new("init").about("Init something");
        let cmd = Command::new("myapp")
            .subcommand(init);
        let matches = cmd.parse(&["init".to_string()]).unwrap();
        let (name, _sub) = matches.subcommand().unwrap();
        assert_eq!(name, "init");
    }
}