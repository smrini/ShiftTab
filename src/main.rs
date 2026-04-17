use crossterm::event::{read, Event, KeyCode};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::execute;
use std::io::{stdout, Write};

fn main() -> anyhow::Result<()> {
    enable_raw_mode()?;

    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, Hide)?;

    // --- NEW: Application State ---
    // A string to hold whatever the user types
    let mut search_query = String::new();
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
        execute!(stdout, Clear(ClearType::All), MoveTo(0, 0))?;
        
        // Draw the Search Box
        println!("> {}\r", search_query);
        println!("--------------------\r");

        // Draw the list of completions
        // NEW: We filter the list to only include items that contain the user's search query
        for item in completions.iter().filter(|c| c.contains(&search_query)) {
            println!("  {}\r", item);
        }
        
        stdout.flush()?;

        // 2. The Input Phase
        let event = read()?; 

        // 3. The Update Phase
        if let Event::Key(key_event) = event {
            match key_event.code {
                // Exit gracefully
                KeyCode::Esc => break,
                
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
    execute!(stdout, Show, LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}