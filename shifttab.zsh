# shifttab.zsh - The bridge between ZLE (Zsh Line Editor) and our Rust binary

function _shifttab_widget() {
    # 1. Ask Rust: "Here is my whole line buffer. Tell me what it SHOULD be."
    local new_buffer=$(/home/dev/ShiftTab/target/debug/ShiftTab "$LBUFFER" </dev/tty)

    # 2. If the user didn't cancel (the output isn't empty)
    if [[ -n "$new_buffer" ]]; then
        # Replace the entire line buffer with Rust's beautifully formatting output
        LBUFFER="$new_buffer"
    fi

    # 3. Tell Zsh to redraw the prompt line
    zle reset-prompt
}

# Turn our function into a real ZLE widget
zle -N _shifttab_widget

# Bind the widget to Shift+Tab (in many terminals this maps to ^[[Z)
bindkey '^[[Z' _shifttab_widget