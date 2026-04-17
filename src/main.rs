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
    // 1. Enter alternate screen and hide the blinking cursor
    execute!(stdout, EnterAlternateScreen, Hide)?;

    // MAIN GAME/TUI LOOP
    loop {
        // 2. The Render Phase: Wipe the screen and move the invisible cursor to the top-left
        execute!(stdout, Clear(ClearType::All), MoveTo(0, 0))?;
        
        println!("=== ShiftTab Pre-Alpha ===\r");
        println!("UI rendered successfully!\r");
        println!("Press 'q' to exit.\r");
        stdout.flush()?;

        // 3. The Input Phase: Wait for the user to press a key
        let event = read()?; 

        if let Event::Key(key_event) = event {
            // Exit if they press Escape or 'q'
            if key_event.code == KeyCode::Esc || key_event.code == KeyCode::Char('q') {
                break;
            }
        }
    }

    // 4. Show the cursor again before returning to the normal terminal
    execute!(stdout, Show, LeaveAlternateScreen)?;
    
    disable_raw_mode()?;
    Ok(())
}