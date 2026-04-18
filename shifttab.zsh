# shifttab.zsh - The bridge between ZLE (Zsh Line Editor) and our Rust binary

function _shifttab_widget() {
    # 1. Run the Rust binary and pass what the user has typed ($LBUFFER) as the first argument
    # We still capture its pure text output
    local selected_flag=$(/home/dev/ShiftTab/target/debug/ShiftTab "$LBUFFER" </dev/tty)

    # 2. If the user picked something (didn't just press Escape),
    # insert a space and the new text into the current line buffer!
    if [[ -n "$selected_flag" ]]; then
        # LBUFFER represents the text to the LEFT of the user's cursor
        LBUFFER+=" $selected_flag"
    fi

    # 3. Tell Zsh to redraw the prompt line to show the new text
    zle reset-prompt
}

# Turn our function into a real ZLE widget
zle -N _shifttab_widget

# Bind the widget to Shift+Tab (in many terminals this maps to ^[[Z)
bindkey '^[[Z' _shifttab_widget