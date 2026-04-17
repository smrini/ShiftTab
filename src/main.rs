use crossterm::event::{read, Event, KeyCode};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::execute;
use std::io::{stdout, Write};

fn main() -> anyhow::Result<()> {
    enable_raw_mode()?;

    let mut stdout = stdout();
    // 1. Tell the terminal to switch to a temporary "alternate" screen
    execute!(stdout, EnterAlternateScreen)?;

    println!("We are now in the Alternate Screen!\r");
    println!("Press 'q' to exit and see what happens.\r");
    stdout.flush()?;

    loop {
        let event = read()?; // Blocks until an event occurs

        if let Event::Key(key_event) = event {
            println!("You pressed: {:?}\r", key_event.code);

            // Exit if they press Escape or 'q'
            if key_event.code == KeyCode::Esc || key_event.code == KeyCode::Char('q') {
                break;
            }
        }
    }

    // 2. Switch back to the main terminal screen before exiting
    execute!(stdout, LeaveAlternateScreen)?;
    
    disable_raw_mode()?;
    Ok(())
}
