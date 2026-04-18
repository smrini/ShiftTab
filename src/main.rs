use crossterm::event::{read, Event, KeyCode, KeyModifiers};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::cursor::{Hide, MoveTo, MoveUp, RestorePosition, SavePosition, Show};
use crossterm::style::{Color, ResetColor, SetBackgroundColor, SetForegroundColor};
use crossterm::execute;
use serde::Deserialize;
use std::io::{stderr, Write};

// --- DEFAULT: Catppuccin Mocha Color Palette ---
const DEFAULT_BASE: (u8, u8, u8) = (36, 39, 58);
const DEFAULT_TEXT: (u8, u8, u8) = (202, 211, 245);
const DEFAULT_HIGHLIGHT: (u8, u8, u8) = (198, 160, 246);
const DEFAULT_BORDER: (u8, u8, u8) = (73, 77, 100);

#[derive(Deserialize, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
enum Mode {
    Compact,
    Extended,
}

impl Default for Mode {
    fn default() -> Self {
        Mode::Extended
    }
}

#[derive(Deserialize, Debug)]
#[serde(default)]
struct ColorConfig {
    #[serde(default = "default_base")]
    base: (u8, u8, u8),
    #[serde(default = "default_text")]
    text: (u8, u8, u8),
    #[serde(default = "default_highlight")]
    highlight: (u8, u8, u8),
    #[serde(default = "default_border")]
    border: (u8, u8, u8),
}

fn default_base() -> (u8, u8, u8) { DEFAULT_BASE }
fn default_text() -> (u8, u8, u8) { DEFAULT_TEXT }
fn default_highlight() -> (u8, u8, u8) { DEFAULT_HIGHLIGHT }
fn default_border() -> (u8, u8, u8) { DEFAULT_BORDER }

impl Default for ColorConfig {
    fn default() -> Self {
        ColorConfig {
            base: DEFAULT_BASE,
            text: DEFAULT_TEXT,
            highlight: DEFAULT_HIGHLIGHT,
            border: DEFAULT_BORDER,
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(default)]
struct KeyConfig {
    #[serde(default = "default_up")]
    up: String,
    #[serde(default = "default_down")]
    down: String,
    #[serde(default = "default_modifier")]
    modifier: String,  // "ctrl", "alt", or "none"
    #[serde(default = "default_vim_top")]
    vim_top: String,  // "g"
    #[serde(default = "default_vim_bottom")]
    vim_bottom: String,  // "G"
    #[serde(default = "default_vim_search")]
    vim_search: String,  // "/"
}

fn default_up() -> String { "k".to_string() }
fn default_down() -> String { "j".to_string() }
fn default_modifier() -> String { "ctrl".to_string() }
fn default_vim_top() -> String { "g".to_string() }
fn default_vim_bottom() -> String { "G".to_string() }
fn default_vim_search() -> String { "/".to_string() }

impl Default for KeyConfig {
    fn default() -> Self {
        KeyConfig {
            up: "k".to_string(),
            down: "j".to_string(),
            modifier: "ctrl".to_string(),
            vim_top: "g".to_string(),
            vim_bottom: "G".to_string(),
            vim_search: "/".to_string(),
        }
    }
}

#[derive(Deserialize, Default, Debug)]
#[serde(default)]
struct Config {
    mode: Mode,
    #[serde(default)]
    colors: ColorConfig,
    #[serde(default)]
    keys: KeyConfig,
}

// --- CONFIG VALIDATION ---
fn validate_config(config: &Config) -> anyhow::Result<()> {
    // Validate modifier key
    match config.keys.modifier.as_str() {
        "ctrl" | "alt" | "none" => {},
        _ => return Err(anyhow::anyhow!(
            "Invalid modifier key '{}'. Must be one of: 'ctrl', 'alt', 'none'\n\
             Check ~/.config/shifttab/config.toml [keys] section",
            config.keys.modifier
        )),
    }
    
    // Validate mode
    if config.mode != Mode::Extended && config.mode != Mode::Compact {
        return Err(anyhow::anyhow!(
            "Invalid mode. Must be 'extended' or 'compact'\n\
             Check ~/.config/shifttab/config.toml mode setting"
        ));
    }
    
    // Validate that key bindings aren't empty
    if config.keys.up.is_empty() {
        return Err(anyhow::anyhow!("Invalid config: 'up' key binding cannot be empty"));
    }
    if config.keys.down.is_empty() {
        return Err(anyhow::anyhow!("Invalid config: 'down' key binding cannot be empty"));
    }
    if config.keys.vim_search.is_empty() {
        return Err(anyhow::anyhow!("Invalid config: 'vim_search' key binding cannot be empty"));
    }
    
    Ok(())
}

fn main() -> anyhow::Result<()> {
    // Default configuration content
    const DEFAULT_CONFIG: &str = r#"# ShiftTab Configuration File
# Location: ~/.config/shifttab/config.toml

# Display mode: "extended" (full-screen) or "compact" (inline)
mode = "extended"

# Color customization (RGB values 0-255)
# Default theme: Catppuccin Mocha
[colors]
base = [36, 39, 58]           # Background color
text = [202, 211, 245]        # Text color
highlight = [198, 160, 246]   # Highlight/selected item color
border = [73, 77, 100]        # Border/separator color

# Keybinding customization
# Navigation uses modifiers (Ctrl or Alt) so you can type normally
# For example: Ctrl+K moves up, plain 'k' is just a regular character
[keys]
up = "k"                       # Move up when pressed with the modifier key
down = "j"                     # Move down when pressed with the modifier key
modifier = "ctrl"              # Modifier for navigation: "ctrl", "alt", or "none"

# Vim-style navigation (always available)
vim_top = "g"                  # Go to top of list (press twice: gg)
vim_bottom = "G"               # Go to bottom of list
vim_search = "/"               # Enter search mode

# When modifier = "ctrl": Use Ctrl+K to navigate up, Ctrl+J to navigate down
# When modifier = "alt":  Use Alt+K to navigate up, Alt+J to navigate down
# When modifier = "none": hjkl always navigate (same as old behavior)
# Arrow keys always work for navigation
# Selection: Enter always selects
# Exit: Escape always exits
"#;

    // --- Step 14: Load Configuration File ---
    let mut config = Config::default();
    if let Some(proj_dirs) = directories::ProjectDirs::from("", "", "shifttab") {
        let config_dir = proj_dirs.config_dir();
        let _ = std::fs::create_dir_all(config_dir);
        let config_file = config_dir.join("config.toml");

        if config_file.exists() {
            if let Ok(contents) = std::fs::read_to_string(&config_file) {
                // If file is empty, populate it with defaults
                if contents.trim().is_empty() {
                    let _ = std::fs::write(&config_file, DEFAULT_CONFIG);
                } else {
                    match toml::from_str::<Config>(&contents) {
                        Ok(parsed) => config = parsed,
                        Err(e) => {
                            return Err(anyhow::anyhow!(
                                "Failed to parse config file at {}:\n{}\n\nConfig file has invalid TOML syntax",
                                config_file.display(),
                                e
                            ));
                        }
                    }
                }
            }
        } else {
            // File doesn't exist, create it with defaults
            let _ = std::fs::write(&config_file, DEFAULT_CONFIG);
        }
    }

    // Validate the loaded config
    validate_config(&config)?;

    enable_raw_mode()?;

    let mut stderr = stderr();
    
    // Only enter the Alternate Screen if we are in Extended Mode.
    // (Compact Mode will just print below the current cursor)
    if config.mode == Mode::Extended {
        execute!(stderr, EnterAlternateScreen)?;
    } else {
        // Reserve 13 lines for the UI (1 for padding + 2 for header + 10 max items)
        // We print newlines to make sure the terminal scrolls if we're at the very bottom
        for _ in 0..13 {
            write!(stderr, "\r\n")?;
        }
        // We moved down 13 lines. Now we move up 11 lines, which parks our cursor 
        // exactly 2 lines below the user's prompt. This naturally creates an empty padding line!
        execute!(stderr, MoveUp(11), SavePosition)?;
    }
    execute!(stderr, Hide)?;

    // --- NEW: Context Parsing ---
    // 1. Read command line arguments passed from Zsh
    let args: Vec<String> = std::env::args().collect();
    
    // 2. The first argument (index 1) is whatever the user has typed so far (the LBUFFER)
    let user_buffer = args.get(1).map(String::as_str).unwrap_or("");
    
    // 3. Find the actual base command! Handle "wrappers" cleverly.
    let wrappers = ["sudo", "doas", "time", "watch", "env", "xargs"];
    let tokens: Vec<&str> = user_buffer.split_whitespace().collect();
    
    let mut base_command = "";
    let mut base_cmd_index = 0;
    
    for (i, &word) in tokens.iter().enumerate() {
        if wrappers.contains(&word) {
            // It's a wrapper! We tentatively mark it as our base_command,
            // but we keep looping to try and find a BETTER real command.
            base_command = word;
            base_cmd_index = i;
        } else if word.starts_with('-') {
            // If we hit a flag (like "-u"), we stop looking. 
            // Whatever command we found previously (even if it was 'sudo') is our final target.
            break;
        } else {
            // We found a concrete, non-wrapper target command!
            base_command = word;
            base_cmd_index = i;
            break;
        }
    }

    // 4. Find out if they are currently midway through typing a word
    // If the buffer DOESN'T end in a space, they are typing a partial word (e.g. "cargo b")
    // If it DOES end in a space (e.g. "cargo "), they are waiting to start a fresh word.
    let is_typing_partial_word = !user_buffer.is_empty() && !user_buffer.ends_with(char::is_whitespace);

    // --- Application State ---
    // If they are typing a partial word, BUT that word is the base command itself (e.g. "sudo mkdir" without a space),
    // we don't want to use "mkdir" as the search query for flags! We only filter if they are typing an argument AFTER the command.
    let mut search_query = String::new();
    if is_typing_partial_word && tokens.len() > base_cmd_index + 1 {
        search_query = tokens.last().unwrap_or(&"").to_string();
    }

    // Keep track of which item is currently highlighted
    let mut selected_index: usize = 0;
    // Multi-select mode: track multiple selected items
    let mut selected_items: std::collections::HashSet<usize> = std::collections::HashSet::new();
    let mut multi_select_mode = false;
    // Store out final choice so we can use it after the UI closes
    let mut final_selection: Option<String> = None;

    // 4. Scrape dynamic completions by running `<base_command> --help`!
    // We now store a tuple of (Flag, Description, Score)
    let mut completions: Vec<(String, String, usize)> = Vec::new();
    
    if !base_command.is_empty() {
        // --- NEW: Persistent Caching Layer ---
        // We use the `directories` crate to find the officially sanctioned cache folder for the current OS.
        // On Linux, this is typically `~/.cache/shifttab/`. 
        let cache_path = if let Some(proj_dirs) = directories::ProjectDirs::from("", "", "shifttab") {
            let cache_dir = proj_dirs.cache_dir();
            
            // Programs are responsible for making sure their own cache folders actually exist before writing to them!
            let _ = std::fs::create_dir_all(cache_dir);
            
            // Build the final path: e.g. ~/.cache/shifttab/mkdir.txt
            cache_dir.join(format!("{}.txt", base_command))
        } else {
            // Absolute fallback if the operating system has completely lost track of the user's home directory
            std::env::temp_dir().join(format!("shifttab_cache_{}.txt", base_command))
        };

        if cache_path.exists() {
            // CACHE HIT: Read directly from the file!
            if let Ok(cached_data) = std::fs::read_to_string(&cache_path) {
                for line in cached_data.lines() {
                    let parts: Vec<&str> = line.splitn(3, '\t').collect();
                    if parts.len() >= 2 {
                        let score = parts.get(2).and_then(|s| s.parse::<usize>().ok()).unwrap_or(0);
                        completions.push((parts[0].to_string(), parts[1].to_string(), score));
                    } else if parts.len() == 1 {
                        completions.push((parts[0].to_string(), String::new(), 0));
                    }
                }
            }
        }
        
        // CACHE MISS (or cache was empty): We must run the expensive background system process
        if completions.is_empty() {
            if let Ok(output) = std::process::Command::new(base_command).arg("--help").output() {
                let help_text_raw = String::from_utf8_lossy(&output.stdout);
                let stripped = strip_ansi_escapes::strip(&*help_text_raw);
                let help_text = String::from_utf8_lossy(&stripped).into_owned();
                
                let mut current_flag: Option<String> = None;
                let mut current_desc = String::new();
                
                for line in help_text.lines() {
                    let trimmed = line.trim_start();
                    
                    // Check if this line starts with a flag (short or long)
                    if trimmed.starts_with('-') {
                        // NEW FLAG FOUND! Save the previous one first.
                        if let Some(f) = current_flag.take() {
                            if !completions.iter().any(|(existing_f, _, _)| existing_f == &f) {
                                completions.push((f, current_desc.trim().to_string(), 0));
                            }
                        }
                        
                        // Now parse ALL the flags on THIS line and combine them intelligently!
                        // Pattern: "-a, --all" or just "-a" or just "--all"
                        let mut flags_on_line = Vec::new();
                        let mut desc_start_pos = 0;
                        
                        // Tokenize by whitespace (using trimmed line for consistency)
                        let tokens: Vec<&str> = trimmed.split_whitespace().collect();
                        
                        // Collect all consecutive tokens that are flags (start with -)
                        for (idx, token) in tokens.iter().enumerate() {
                            // Strip trailing commas
                            let clean = token.trim_end_matches(',');
                            
                            // Valid flag: starts with - and is either 2 chars (-X) or starts with --
                            if clean.starts_with('-') && (clean.len() == 2 || clean.starts_with("--")) {
                                flags_on_line.push(clean.to_string());
                                desc_start_pos = idx + 1;
                            } else {
                                // Hit a non-flag token, stop collecting
                                break;
                            }
                        }
                        
                        // Combine flags smartly: "-a (--all)" if multiple, or just the flag if single
                        let combined_flag = if flags_on_line.len() > 1 {
                            format!("{} ({})", flags_on_line[0], flags_on_line[1..].join(", "))
                        } else if !flags_on_line.is_empty() {
                            flags_on_line[0].clone()
                        } else {
                            continue; // No valid flags found on this line
                        };
                        
                        current_flag = Some(combined_flag);
                        
                        // Extract description from remaining tokens on this line
                        current_desc = if desc_start_pos < tokens.len() {
                            tokens[desc_start_pos..].join(" ")
                        } else {
                            String::new()
                        };
                    } else if current_flag.is_some() {
                        // This line continues the description of the current flag
                        if !trimmed.is_empty() {
                            if !current_desc.is_empty() {
                                current_desc.push(' ');
                            }
                            current_desc.push_str(trimmed);
                        }
                    }
                }
                
                // Don't forget to push the very last flag
                if let Some(f) = current_flag {
                    if !completions.iter().any(|(existing_f, _, _)| existing_f == &f) {
                        completions.push((f, current_desc.trim().to_string(), 0));
                    }
                }

                // Write the results to the cache file so we never have to run `--help` for this command again!
                // We use Tab (\t) to separate Flag, Description, and Score
                let cache_contents: Vec<String> = completions.iter().map(|(f, d, s)| format!("{}\t{}\t{}", f, d, s)).collect();
                let _ = std::fs::write(cache_path, cache_contents.join("\n"));
            }
        }
    }

    // Fallback if the command didn't have a `--help` or we couldn't parse it
    if completions.is_empty() {
        completions.push(("--help".to_string(), "Show help menu".to_string(), 0));
        completions.push(("--version".to_string(), "Show version info".to_string(), 0));
    }

    // MAIN GAME/TUI LOOP
    // Convert config colors to crossterm Color objects
    let color_base = Color::Rgb { r: config.colors.base.0, g: config.colors.base.1, b: config.colors.base.2 };
    let color_text = Color::Rgb { r: config.colors.text.0, g: config.colors.text.1, b: config.colors.text.2 };
    let color_highlight = Color::Rgb { r: config.colors.highlight.0, g: config.colors.highlight.1, b: config.colors.highlight.2 };
    let color_border = Color::Rgb { r: config.colors.border.0, g: config.colors.border.1, b: config.colors.border.2 };
    
    let mut last_vim_top_time = std::time::Instant::now();
    let vim_double_tap_timeout = std::time::Duration::from_millis(300);
    
    loop {
        // 1. The Render Phase
        if config.mode == Mode::Extended {
            execute!(
                stderr, 
                SetBackgroundColor(color_base),
                SetForegroundColor(color_text),
                MoveTo(0, 0)
            )?;
            // clear the first line and then let the rest of the loop overwrite everything else.
            execute!(stderr, Clear(ClearType::UntilNewLine))?;
            write!(stderr, "\r\n")?;
        } else {
            execute!(
                stderr, 
                RestorePosition, // Go back to the top of our 12-line reserved block
                ResetColor,      // RESET COLOR SO CLEAR DOESN'T BLEED TO THE BOTTOM OF THE TERMINAL
                Clear(ClearType::FromCursorDown),
                SetBackgroundColor(color_base),
                SetForegroundColor(color_text)
            )?;
        }
        
        // Draw the Search Box (styled!)
        execute!(stderr, SetForegroundColor(color_highlight))?;
        write!(stderr, "> ")?;
        execute!(stderr, SetForegroundColor(color_text), Clear(ClearType::UntilNewLine))?;
        write!(stderr, "{}\r\n", search_query)?;
        
        execute!(stderr, SetForegroundColor(color_border), Clear(ClearType::UntilNewLine))?;
        write!(stderr, "--------------------\r\n")?;

        // Sort completions by score (highest first)
        completions.sort_by(|a, b| b.2.cmp(&a.2));

        // Prepare the filtered list (Note: completions is now a Vec with score)
        let filtered: Vec<&(String, String, usize)> = completions
            .iter()
            .filter(|c| c.0.contains(&search_query))
            .collect();

        // Ensure our selection doesn't go out of bounds if the list shrinks
        if filtered.is_empty() {
            selected_index = 0;
        } else if selected_index >= filtered.len() {
            selected_index = filtered.len() - 1;
        }

        // Setup pagination/scrolling for our list
        // In extended mode, use full terminal height minus space for top padding, search box, separator, help bar, and status bar
        // In compact mode, limit to 10 rows
        let (_, rows) = crossterm::terminal::size().unwrap_or((80, 24));
        let max_visible_items = if config.mode == Mode::Extended {
            // Reserve 1 row for top padding + 2 rows for search box and separator + 1 row for help bar + 1 row for status bar
            (rows as usize).saturating_sub(5).max(3)  // At least 3 rows
        } else {
            10  // Compact mode keeps 10 rows
        };
        let mut start_idx = 0;
        
        // If we are currently selecting an item beyond our window, scroll the window!
        if selected_index >= max_visible_items {
            start_idx = selected_index - max_visible_items + 1;
        }
        
        // Take only our visible window's worth of items. Collect them to make math easier.
        let visible_items: Vec<_> = filtered.iter().enumerate().skip(start_idx).take(max_visible_items).collect();

        // Let's figure out how wide the terminal is to build our pane split
        let (cols, _) = crossterm::terminal::size().unwrap_or((80, 24));
        
        // In compact mode, use more space for the description; in extended mode, use 50/50 split
        let left_pane_width = if config.mode == Mode::Extended {
            (cols / 2).saturating_sub(3) as usize
        } else {
            // Compact mode: narrow left pane (just enough for flag + prefix)
            25
        };
        let right_pane_width = if config.mode == Mode::Extended {
            (cols / 2).saturating_sub(2) as usize
        } else {
            // Compact mode: use most of remaining width for description
            (cols as usize).saturating_sub(left_pane_width + 5)
        };

        // Get the description of the CURRENTLY SELECTED item so we can wrap it over multiple lines
        let selected_desc = if let Some(sel) = filtered.get(selected_index) {
            &sel.1
        } else {
            ""
        };

        // Basic Word Wrap logic: split the description text so it fits securely inside the right_pane_width
        // Using .chars().count() to properly handle Unicode characters, not bytes
        let mut desc_lines = Vec::new();
        let mut current_line = String::new();
        for word in selected_desc.split_whitespace() {
            let word_char_count = word.chars().count();
            let current_line_char_count = current_line.chars().count();
            let space_needed = if current_line.is_empty() { 0 } else { 1 };
            
            // If adding this word would exceed the pane width, start a new line
            if current_line_char_count + word_char_count + space_needed > right_pane_width {
                desc_lines.push(current_line.clone());
                current_line = word.to_string();
                
                // If the single word itself exceeds the width, truncate it
                if word_char_count > right_pane_width {
                    current_line = word.chars().take(right_pane_width).collect();
                }
            } else {
                if !current_line.is_empty() { current_line.push(' '); }
                current_line.push_str(word);
            }
        }
        if !current_line.is_empty() { desc_lines.push(current_line); }

        // Draw the split UI! We draw as many rows as available in the terminal.
        for row in 0..max_visible_items {
            // --- LEFT PANE (The Flags) ---
            let (is_selected, left_text) = if row < visible_items.len() {
                let (i, item) = visible_items[row];
                let selected = i == selected_index;
                let is_multi_selected = selected_items.contains(&i);
                
                // Build prefix based on mode and selection state
                let prefix = if multi_select_mode {
                    // In multi-select mode, show checkbox
                    if is_multi_selected {
                        " ✓ " // Checkmark for selected
                    } else {
                        " ☐ " // Empty box for unselected
                    }
                } else if selected {
                    // Single select mode, show arrow for selected
                    " ▶ "
                } else {
                    "   "
                };
                
                // Truncate the flag if it is longer than our left pane
                // Use .chars().count() to properly handle Unicode characters, not bytes
                let mut flag_text = item.0.clone();
                let max_flag_len = left_pane_width.saturating_sub(prefix.chars().count());
                let flag_char_count = flag_text.chars().count();
                if flag_char_count > max_flag_len {
                    flag_text = flag_text.chars().take(max_flag_len).collect();
                }
                
                // Pad it out with exact spaces so the background color is a perfect rectangle
                (selected, format!("{}{:<width$}", prefix, flag_text, width = max_flag_len))
            } else {
                // If there are fewer than 10 items, just fill the rest of the left pane with empty space
                (false, format!("{:<width$}", "", width = left_pane_width))
            };

            // --- RIGHT PANE (The Description) ---
            let right_text = if row < desc_lines.len() {
                let mut chunk = desc_lines[row].clone();
                // Use .chars().count() to properly handle Unicode characters, not bytes
                let chunk_char_count = chunk.chars().count();
                if chunk_char_count > right_pane_width {
                    chunk = chunk.chars().take(right_pane_width).collect();
                }
                format!(" {:<width$}", chunk, width = right_pane_width - 1)
            } else {
                format!("{:<width$}", "", width = right_pane_width)
            };

            // --- RENDER THE ROW ---
            if is_selected {
                // Left Pane highlighted
                execute!(stderr, SetBackgroundColor(color_highlight), SetForegroundColor(color_base))?;
                write!(stderr, "{}", left_text)?;
                
                // The middle separator line
                execute!(stderr, SetBackgroundColor(color_base), SetForegroundColor(color_border))?;
                write!(stderr, " │")?;
                
                // Right Pane standard colors
                execute!(stderr, SetForegroundColor(color_text))?;
                write!(stderr, "{}", right_text)?;
                execute!(stderr, Clear(ClearType::UntilNewLine))?;
                write!(stderr, "\r\n")?;
            } else {
                // Standard row colors across the board
                execute!(stderr, SetBackgroundColor(color_base), SetForegroundColor(color_text))?;
                write!(stderr, "{}", left_text)?;
                
                // The middle separator line
                execute!(stderr, SetForegroundColor(color_border))?;
                write!(stderr, " │")?;
                
                // Right Pane standard colors
                execute!(stderr, SetForegroundColor(color_text))?;
                write!(stderr, "{}", right_text)?;
                execute!(stderr, Clear(ClearType::UntilNewLine))?;
                write!(stderr, "\r\n")?;
            }
        }
        
        // --- HELP BAR (Extended Mode Only) ---
        if config.mode == Mode::Extended {
            // Build dynamic help text based on config
            let nav_keys = if config.keys.modifier == "none" {
                format!("[↑↓/{}{}]", config.keys.up, config.keys.down)
            } else {
                let mod_str = if config.keys.modifier == "ctrl" { "^" } else { "Alt+" };
                format!("[↑↓/{}{}/{}{}]", config.keys.up, config.keys.down, mod_str, config.keys.up)
            };
            
            // Build the actions part dynamically from config
            let help_text = format!(
                "{} navigate | [{}] search | [Space] toggle | [Ctrl+Space] multi | [{}{}] jump | [Enter] select | [Esc] exit",
                nav_keys,
                config.keys.vim_search,
                config.keys.vim_top,
                config.keys.vim_bottom
            );
            
            // Get terminal width
            let (cols, _) = crossterm::terminal::size().unwrap_or((80, 24));
            let max_text_width = (cols as usize).saturating_sub(4); // Leave 2 char margin on each side
            
            // Word-wrap the help text to fit within max_text_width
            let mut help_lines = Vec::new();
            let mut current_line = String::new();
            
            for word in help_text.split_whitespace() {
                let word_len = word.chars().count();
                let current_len = current_line.chars().count();
                let space_needed = if current_line.is_empty() { 0 } else { 1 };
                
                if current_len + word_len + space_needed > max_text_width && !current_line.is_empty() {
                    help_lines.push(current_line.clone());
                    current_line = word.to_string();
                } else {
                    if !current_line.is_empty() {
                        current_line.push(' ');
                    }
                    current_line.push_str(word);
                }
            }
            if !current_line.is_empty() {
                help_lines.push(current_line);
            }
            
            // Render each line centered
            execute!(stderr, SetForegroundColor(color_border), Clear(ClearType::UntilNewLine))?;
            for (idx, line) in help_lines.iter().enumerate() {
                let text_width = line.chars().count();
                let padding = if text_width < cols as usize {
                    (cols as usize - text_width) / 2
                } else {
                    0
                };
                write!(stderr, "{}{}", " ".repeat(padding), line)?;
                execute!(stderr, Clear(ClearType::UntilNewLine))?;
                if idx < help_lines.len() - 1 {
                    write!(stderr, "\r\n")?;
                }
            }
            write!(stderr, "\r\n")?;
        }
        
        // --- STATUS BAR ---
        if !filtered.is_empty() {
            let (cols, _) = crossterm::terminal::size().unwrap_or((80, 24));
            
            // Build status text with multi-select info
            let status_text = if multi_select_mode {
                if selected_items.is_empty() {
                    format!("MULTI | {}/{}", selected_index + 1, filtered.len())
                } else {
                    format!("MULTI ({}) | {}/{}", selected_items.len(), selected_index + 1, filtered.len())
                }
            } else {
                format!("{}/{}", selected_index + 1, filtered.len())
            };
            
            if config.mode == Mode::Extended {
                // Build scrollbar for extended mode (spanning full width)
                let scrollbar_width = (cols as usize).saturating_sub(status_text.len() + 5);
                let filled = if filtered.len() <= 1 { 
                    scrollbar_width 
                } else {
                    (selected_index * scrollbar_width) / (filtered.len() - 1)
                };
                let scrollbar = format!("{}{}",
                    "█".repeat(filled.min(scrollbar_width)),
                    "░".repeat(scrollbar_width.saturating_sub(filled))
                );
                
                // Render status bar (no newline - it's the last row)
                execute!(stderr, SetForegroundColor(color_border))?;
                write!(stderr, " {} ", scrollbar)?;
                execute!(stderr, SetForegroundColor(color_text))?;
                write!(stderr, "[{}]", status_text)?;
            } else {
                // Compact mode: show scrollbar spanning full width + status counter
                let scrollbar_width = (cols as usize).saturating_sub(status_text.len() + 5);  // -6 for spacing and brackets
                let filled = if filtered.len() <= 1 { 
                    scrollbar_width 
                } else {
                    (selected_index * scrollbar_width) / (filtered.len() - 1)
                };
                let scrollbar = format!("{}{}",
                    "█".repeat(filled.min(scrollbar_width)),
                    "░".repeat(scrollbar_width.saturating_sub(filled))
                );
                
                execute!(stderr, SetForegroundColor(color_border))?;
                write!(stderr, " {} ", scrollbar)?;
                execute!(stderr, SetForegroundColor(color_text))?;
                write!(stderr, "[{}] ", status_text)?;
            }
        }
        
        // Reset colors before moving on, to make sure nothing weird happens with trailing artifacts
        execute!(stderr, ResetColor)?;
        stderr.flush()?;

        // 2. The Input Phase
        let event = read()?; 

        // 3. The Update Phase
        if let Event::Key(key_event) = event {
            match key_event.code {
                // Exit gracefully - ESC always works
                KeyCode::Esc => break,
                
                // Arrow keys and Ctrl bindings always work for navigation
                KeyCode::Up => {
                    selected_index = selected_index.saturating_sub(1);
                }

                KeyCode::Down => {
                    if !filtered.is_empty() && selected_index < filtered.len() - 1 {
                        selected_index += 1;
                    }
                }

                // Enter always selects (as separate key, not as char)
                KeyCode::Enter => {
                    let mut selected_flags = Vec::new();
                    
                    if multi_select_mode && !selected_items.is_empty() {
                        // Multi-select mode: combine all selected items
                        let mut selections = vec![];
                        let mut indices: Vec<_> = selected_items.iter().copied().collect();
                        indices.sort();
                        for idx in indices {
                            if let Some((flag, _, _)) = filtered.get(idx) {
                                selections.push(flag.clone());
                                selected_flags.push(flag.clone());
                            }
                        }
                        final_selection = Some(selections.join(" "));
                    } else if let Some(selected_item) = filtered.get(selected_index) {
                        // Single-select mode: just select current item
                        let selected_flag = selected_item.0.clone();
                        final_selection = Some(selected_flag.clone());
                        selected_flags.push(selected_flag);
                    }
                    
                    // Update scores for selected flags (outside of borrow scope)
                    for flag in selected_flags {
                        if let Some(entry) = completions.iter_mut().find(|(f, _, _)| f == &flag) {
                            entry.2 += 1;
                        }
                    }
                    
                    break;
                }

                // Space: toggle multi-select on current item (only in multi-select mode), or Ctrl+Space to enter/exit mode
                KeyCode::Char(' ') => {
                    // Check for Ctrl+Space first
                    if key_event.modifiers.contains(KeyModifiers::CONTROL) {
                        multi_select_mode = !multi_select_mode;
                        // When entering multi-select mode, add current item to selection
                        if multi_select_mode {
                            selected_items.insert(selected_index);
                        } else {
                            // When exiting multi-select mode, clear selections
                            selected_items.clear();
                        }
                    } else if multi_select_mode {
                        // Space without Ctrl: toggle current item in multi-select mode
                        if selected_items.contains(&selected_index) {
                            selected_items.remove(&selected_index);
                        } else {
                            selected_items.insert(selected_index);
                        }
                    } else {
                        // Not in multi-select mode, space just appends to search
                        search_query.push(' ');
                    }
                    continue;
                }

                // Context-aware character input with modifier support
                KeyCode::Char(c) => {
                    let c_str = c.to_string();
                    
                    // Check if the required modifier is pressed
                    let modifier_matches = match config.keys.modifier.as_str() {
                        "ctrl" => key_event.modifiers.contains(KeyModifiers::CONTROL),
                        "alt" => key_event.modifiers.contains(KeyModifiers::ALT),
                        "none" => true,  // No modifier required
                        _ => false,      // Unknown modifier, treat as no match
                    };
                    
                    // --- VIM KEYBINDINGS (require modifier) ---
                    // "/" enters search mode (clears current search)
                    if c_str == config.keys.vim_search && modifier_matches {
                        search_query.clear();
                        continue;
                    }
                    
                    // "G" goes to bottom
                    if c_str == config.keys.vim_bottom && modifier_matches {
                        if !filtered.is_empty() {
                            selected_index = filtered.len() - 1;
                        }
                        continue;
                    }
                    
                    // "g" goes to top (requires double tap: gg)
                    if c_str == config.keys.vim_top && modifier_matches {
                        let now = std::time::Instant::now();
                        if now.duration_since(last_vim_top_time) < vim_double_tap_timeout {
                            // Double tap detected, go to top
                            selected_index = 0;
                            last_vim_top_time = std::time::Instant::now() - std::time::Duration::from_secs(1); // Reset
                        } else {
                            // First tap, set timer
                            last_vim_top_time = now;
                        }
                        continue;
                    }
                    
                    // Check if this character matches a navigation key
                    let is_up_key = c_str == config.keys.up;
                    let is_down_key = c_str == config.keys.down;
                    
                    // Navigate only if: key matches AND modifier matches
                    if (is_up_key || is_down_key) && modifier_matches {
                        if is_up_key {
                            selected_index = selected_index.saturating_sub(1);
                        } else if is_down_key {
                            if !filtered.is_empty() && selected_index < filtered.len() - 1 {
                                selected_index += 1;
                            }
                        }
                    } else {
                        // Not a navigation key with correct modifier, append to search
                        search_query.push(c);
                    }
                }
                
                // If the user hits backspace, remove the last character
                KeyCode::Backspace => {
                    search_query.pop();
                }
                
                // Ignore all other keys for now
                _ => {}
            }
        }
    }

    // Save updated scores back to cache
    if !base_command.is_empty() {
        if let Some(proj_dirs) = directories::ProjectDirs::from("", "", "shifttab") {
            let cache_dir = proj_dirs.cache_dir();
            let cache_path = cache_dir.join(format!("{}.txt", base_command));
            let cache_contents: Vec<String> = completions.iter().map(|(f, d, s)| format!("{}\t{}\t{}", f, d, s)).collect();
            let _ = std::fs::write(cache_path, cache_contents.join("\n"));
        }
    }

    // Teardown
    if config.mode == Mode::Extended {
        execute!(stderr, LeaveAlternateScreen)?;
    } else {
        // Restore cursor up 1 line to safely clear the padding layer, then move UP 1 more line to return EXACTLY
        // to the user's original command prompt row before handing control back to Zsh!
        execute!(
            stderr, 
            RestorePosition, 
            MoveUp(1), 
            ResetColor, 
            Clear(ClearType::FromCursorDown),
            MoveUp(1)
        )?;
    }
    execute!(stderr, Show)?;
    disable_raw_mode()?;

    // Output the completed user buffer back to the shell!
    if let Some(selection) = final_selection {
        let mut new_buffer = user_buffer.to_string();

        let is_typing_partial_flag = is_typing_partial_word && tokens.len() > base_cmd_index + 1;

        if is_typing_partial_flag {
            // Find the index of the last space they typed before their partial word
            if let Some(last_space_idx) = new_buffer.rfind(char::is_whitespace) {
                // Slice off the partial word so we can inject the cleanly selected item on top
                new_buffer.truncate(last_space_idx + 1);
            } else {
                // If there's no spaces in the whole string, just clear it
                new_buffer.clear();
            }
        } else {
            // If they weren't typing a partial flag (e.g. "ls" or "ls "), we need to add a space 
            // ourselves so the flag doesn't stick to the last argument.
            if !new_buffer.is_empty() && !new_buffer.ends_with(char::is_whitespace) {
                new_buffer.push(' ');
            }
        }
        
        // Extract just the base flags, removing descriptions in parentheses
        // E.g., "-a (--all)" becomes "-a", and "-a (--all) -b (--brief)" becomes "-a -b"
        let cleaned_flags = selection
            .split_whitespace()
            .filter(|part| !part.starts_with('('))  // Skip parenthetical descriptions
            .collect::<Vec<_>>()
            .join(" ");
        
        // Use cleaned flags if we got them, otherwise use original selection
        let output_flag = if cleaned_flags.is_empty() { &selection } else { &cleaned_flags };
        
        // Print the assembled, finalized buffer to stdout!
        // We append a trailing space so the user can immediately type the next argument!
        print!("{}{} ", new_buffer, output_flag);
    }
    
    Ok(())
}