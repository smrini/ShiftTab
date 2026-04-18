use crossterm::event::{read, Event, KeyCode};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::cursor::{Hide, MoveTo, MoveUp, RestorePosition, SavePosition, Show};
use crossterm::style::{Color, ResetColor, SetBackgroundColor, SetForegroundColor};
use crossterm::execute;
use serde::Deserialize;
use std::io::{stderr, Write};

// --- NEW: Catppuccin Mocha Color Palette ---
const MACCHIATO_BASE: Color = Color::Rgb { r: 36, g: 39, b: 58 };
const MACCHIATO_TEXT: Color = Color::Rgb { r: 202, g: 211, b: 245 };
const MACCHIATO_MAUVE: Color = Color::Rgb { r: 198, g: 160, b: 246 };
const MACCHIATO_SURFACE1: Color = Color::Rgb { r: 73, g: 77, b: 100 };

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

#[derive(Deserialize, Default, Debug)]
#[serde(default)]
struct Config {
    mode: Mode,
}

fn main() -> anyhow::Result<()> {
    // --- Step 14: Load Configuration File ---
    let mut config = Config::default();
    if let Some(proj_dirs) = directories::ProjectDirs::from("", "", "shifttab") {
        let config_dir = proj_dirs.config_dir();
        let _ = std::fs::create_dir_all(config_dir);
        let config_file = config_dir.join("config.toml");

        if config_file.exists() {
            if let Ok(contents) = std::fs::read_to_string(&config_file) {
                if let Ok(parsed) = toml::from_str(&contents) {
                    config = parsed;
                }
            }
        } else {
            // Write a default config if none exists to show the user it's there
            let default_toml = "mode = \"extended\"\n";
            let _ = std::fs::write(&config_file, default_toml);
        }
    }

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
    // Store out final choice so we can use it after the UI closes
    let mut final_selection: Option<String> = None;

    // 4. Scrape dynamic completions by running `<base_command> --help`!
    let mut completions: Vec<String> = Vec::new();
    
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
                completions = cached_data.lines().map(|s| s.to_string()).collect();
            }
        }
        
        // CACHE MISS (or cache was empty): We must run the expensive background system process
        if completions.is_empty() {
            if let Ok(output) = std::process::Command::new(base_command).arg("--help").output() {
                let help_text = String::from_utf8_lossy(&output.stdout);
                
                // This Regex looks for `-` followed by a single letter OR `--` followed by a word
                let re = regex::Regex::new(r"(-[a-zA-Z0-9]|--[a-zA-Z0-9\-]+)").unwrap();
                
                for cap in re.captures_iter(&help_text) {
                    let flag = cap[0].to_string();
                    if !completions.contains(&flag) {
                        completions.push(flag); // Save unique flags only
                    }
                }

                // Write the results to the cache file so we never have to run `--help` for this command again!
                let cache_contents = completions.join("\n");
                let _ = std::fs::write(cache_path, cache_contents);
            }
        }
    }

    // Fallback if the command didn't have a `--help` or we couldn't parse it
    if completions.is_empty() {
        completions.push("--help".to_string());
        completions.push("--version".to_string());
    }

    // MAIN GAME/TUI LOOP
    loop {
        // 1. The Render Phase
        if config.mode == Mode::Extended {
            execute!(
                stderr, 
                SetBackgroundColor(MACCHIATO_BASE),
                SetForegroundColor(MACCHIATO_TEXT),
                Clear(ClearType::All), 
                MoveTo(0, 0)
            )?;
        } else {
            execute!(
                stderr, 
                RestorePosition, // Go back to the top of our 12-line reserved block
                ResetColor,      // RESET COLOR SO CLEAR DOESN'T BLEED TO THE BOTTOM OF THE TERMINAL
                Clear(ClearType::FromCursorDown),
                SetBackgroundColor(MACCHIATO_BASE),
                SetForegroundColor(MACCHIATO_TEXT)
            )?;
        }
        
        // Draw the Search Box (styled!)
        execute!(stderr, SetForegroundColor(MACCHIATO_MAUVE))?;
        write!(stderr, "> ")?;
        execute!(stderr, SetForegroundColor(MACCHIATO_TEXT), Clear(ClearType::UntilNewLine))?;
        write!(stderr, "{}\r\n", search_query)?;
        
        execute!(stderr, SetForegroundColor(MACCHIATO_SURFACE1), Clear(ClearType::UntilNewLine))?;
        write!(stderr, "--------------------\r\n")?;

        // Prepare the filtered list (Note: completions is now a Vec<String>)
        let filtered: Vec<&String> = completions
            .iter()
            .filter(|c| c.contains(&search_query))
            .collect();

        // Ensure our selection doesn't go out of bounds if the list shrinks
        if filtered.is_empty() {
            selected_index = 0;
        } else if selected_index >= filtered.len() {
            selected_index = filtered.len() - 1;
        }

        // Setup pagination/scrolling for our list
        let max_visible_items = 10;
        let mut start_idx = 0;
        
        // If we are currently selecting an item beyond our window, scroll the window!
        if selected_index >= max_visible_items {
            start_idx = selected_index - max_visible_items + 1;
        }
        
        // Take only our visible window's worth of items
        let visible_items = filtered.iter().enumerate().skip(start_idx).take(max_visible_items);

        // Draw the list of completions
        for (i, item) in visible_items {
            // We use standard ANSI resets per-line so the background handles cleanly
            if i == selected_index {
                // Highlight the selected item (Mauve BG, Base FG)
                execute!(
                    stderr, 
                    SetBackgroundColor(MACCHIATO_MAUVE),
                    SetForegroundColor(MACCHIATO_BASE),
                    Clear(ClearType::UntilNewLine) // Make sure the background color spans the full terminal width
                )?;
                write!(stderr, " ▶ {} \r\n", item)?;
            } else {
                // Normal item
                execute!(
                    stderr, 
                    SetBackgroundColor(MACCHIATO_BASE),
                    SetForegroundColor(MACCHIATO_TEXT),
                    Clear(ClearType::UntilNewLine) // Make sure the background color spans the full terminal width
                )?;
                write!(stderr, "   {} \r\n", item)?;
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
                // Exit gracefully
                KeyCode::Esc => break,
                
                // Confirm selection
                KeyCode::Enter => {
                    if let Some(selected_item) = filtered.get(selected_index) {
                        final_selection = Some(selected_item.to_string());
                    }
                    break;
                }

                // Navigate up
                KeyCode::Up => {
                    selected_index = selected_index.saturating_sub(1);
                }

                // Navigate down
                KeyCode::Down => {
                    if !filtered.is_empty() && selected_index < filtered.len() - 1 {
                        selected_index += 1;
                    }
                }
                
                // If the user types a normal character, append it to our query
                KeyCode::Char(c) => {
                    search_query.push(c);
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

    // Teardown
    if config.mode == Mode::Extended {
        execute!(stderr, LeaveAlternateScreen)?;
    } else {
        // Restore cursor to original line, move UP 1 line to clear the padding, then clear downwards
        execute!(stderr, RestorePosition, MoveUp(1), ResetColor, Clear(ClearType::FromCursorDown))?;
    }
    execute!(stderr, Show)?;
    disable_raw_mode()?;

    // Output the completed user buffer back to the shell!
    if let Some(selection) = final_selection {
        let mut new_buffer = user_buffer.to_string();

        if is_typing_partial_word {
            // Find the index of the last space they typed before their partial word
            if let Some(last_space_idx) = new_buffer.rfind(char::is_whitespace) {
                // Slice off the partial word so we can inject the cleanly selected item on top
                new_buffer.truncate(last_space_idx + 1);
            } else {
                // If there's no spaces in the whole string, just clear it
                new_buffer.clear();
            }
        } else {
            // If they weren't typing a partial word (e.g. "ls "), we need to add a space 
            // ourselves so the flag doesn't stick to the last argument.
            if !new_buffer.is_empty() && !new_buffer.ends_with(char::is_whitespace) {
                new_buffer.push(' ');
            }
        }
        
        // Print the assembled, finalized buffer to stdout!
        print!("{}{}", new_buffer, selection);
    }
    
    Ok(())
}