use std::io::{self, Write};
use std::path::Path;
use std::process::{Command, Stdio};

use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use crate::config::Config;

pub struct Editor {
    command: String,
    args: Vec<String>,
}

impl Editor {
    pub fn new(config: &Config) -> Self {
        Self {
            command: config.editor.clone(),
            args: config.editor_args.clone(),
        }
    }

    pub fn open(&self, path: &Path) -> Result<(), String> {
        let path_str = path.to_string_lossy().to_string();

        // Restore terminal to normal state
        disable_raw_mode().map_err(|e| format!("Failed to disable raw mode: {}", e))?;
        execute!(io::stdout(), LeaveAlternateScreen)
            .map_err(|e| format!("Failed to leave alternate screen: {}", e))?;

        // Run editor with inherited stdio
        let mut cmd = Command::new(&self.command);
        for arg in &self.args {
            cmd.arg(arg);
        }
        cmd.arg(&path_str);
        cmd.stdin(Stdio::inherit());
        cmd.stdout(Stdio::inherit());
        cmd.stderr(Stdio::inherit());

        let result = match cmd.spawn() {
            Ok(mut child) => match child.wait() {
                Ok(_) => Ok(()),
                Err(e) => Err(format!("Editor process error: {}", e)),
            },
            Err(e) => Err(format!("Failed to open editor '{}': {}", self.command, e)),
        };

        // Restore TUI state
        enable_raw_mode().map_err(|e| format!("Failed to enable raw mode: {}", e))?;
        execute!(io::stdout(), EnterAlternateScreen)
            .map_err(|e| format!("Failed to enter alternate screen: {}", e))?;

        // Force redraw
        io::stdout().flush().ok();

        result
    }
}
