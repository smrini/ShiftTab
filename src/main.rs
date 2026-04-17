use crossterm::event::{read, Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use std::io::{stdout, Write};

fn main() -> anyhow::Result<()> {
    // 1. Put the terminal into "raw mode"
    enable_raw_mode()?;

    println!("Terminal is now in raw mode. Press any key to exit.\r");
    
    // We need to flush stdout because in raw mode, newlines don't automatically flush
    stdout().flush()?;

    // 2. Wait for a single event (like a key press)
    loop {
        let event = read()?; // Blocks until an event occurs

        if let Event::Key(key_event) = event {
            // Print the key we just pressed. 
            // Note the \r\n: in raw mode, \n just moves down, it doesn't return to the left margin!
            println!("You pressed: {:?}\r", key_event.code);

            // Exit if they press Escape or 'q'
            if key_event.code == KeyCode::Esc || key_event.code == KeyCode::Char('q') {
                break;
            }
        }
    }

    // 3. Always restore the terminal before exiting!
    disable_raw_mode()?;
    
    println!("Safely returned to normal mode.");
    Ok(())
}