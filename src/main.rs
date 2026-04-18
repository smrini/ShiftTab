use crossterm::event::{read, Event, KeyCode};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::style::{Color, ResetColor, SetBackgroundColor, SetForegroundColor};
use crossterm::execute;
use std::io::{stderr, Write};

// --- NEW: Catppuccin Mocha Color Palette ---
const MACCHIATO_BASE: Color = Color::Rgb { r: 36, g: 39, b: 58 };
const MACCHIATO_TEXT: Color = Color::Rgb { r: 202, g: 211, b: 245 };
const MACCHIATO_MAUVE: Color = Color::Rgb { r: 198, g: 160, b: 246 };
const MACCHIATO_SURFACE1: Color = Color::Rgb { r: 73, g: 77, b: 100 };

fn main() -> anyhow::Result<()> {
    enable_raw_mode()?;

    let mut stderr = stderr();
    execute!(stderr, EnterAlternateScreen, Hide)?;

    // --- NEW: Application State ---
    // A string to hold whatever the user types
    let mut search_query = String::new();
    // Keep track of which item is currently highlighted
    let mut selected_index: usize = 0;
    // Store out final choice so we can use it after the UI closes
    let mut final_selection: Option<String> = None;

    // A hardcoded mock list of command flags/options
    let completions = vec![
        "--all",
        "--force",
        "--help",
        "--quiet",
        "--verbose",
    ];

    // MAIN GAME/TUI LOOP
    loop {
        // 1. The Render Phase
        execute!(
            stderr, 
            SetBackgroundColor(MACCHIATO_BASE),
            SetForegroundColor(MACCHIATO_TEXT),
            Clear(ClearType::All), 
            MoveTo(0, 0)
        )?;
        
        // Draw the Search Box (styled!)
        execute!(stderr, SetForegroundColor(MACCHIATO_MAUVE))?;
        write!(stderr, "> ")?;
        execute!(stderr, SetForegroundColor(MACCHIATO_TEXT))?;
        write!(stderr, "{}\r\n", search_query)?;
        
        execute!(stderr, SetForegroundColor(MACCHIATO_SURFACE1))?;
        write!(stderr, "--------------------\r\n")?;

        // Prepare the filtered list
        let filtered: Vec<&&str> = completions
            .iter()
            .filter(|c| c.contains(&search_query))
            .collect();

        // Ensure our selection doesn't go out of bounds if the list shrinks
        if filtered.is_empty() {
            selected_index = 0;
        } else if selected_index >= filtered.len() {
            selected_index = filtered.len() - 1;
        }

        // Draw the list of completions
        for (i, item) in filtered.iter().enumerate() {
            // We use standard ANSI resets per-line so the background handles cleanly
            if i == selected_index {
                // Highlight the selected item (Mauve BG, Base FG)
                execute!(
                    stderr, 
                    SetBackgroundColor(MACCHIATO_MAUVE),
                    SetForegroundColor(MACCHIATO_BASE)
                )?;
                write!(stderr, " ▶ {} \r\n", item)?;
            } else {
                // Normal item
                execute!(
                    stderr, 
                    SetBackgroundColor(MACCHIATO_BASE),
                    SetForegroundColor(MACCHIATO_TEXT)
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
    execute!(stderr, Show, LeaveAlternateScreen)?;
    disable_raw_mode()?;

    // Print ONLY the bare result to stdout so the shell can capture it!
    if let Some(selection) = final_selection {
        print!("{}", selection);
    }
    
    Ok(())
}