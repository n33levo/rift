# VHS Tape for Rift Demo
# Run `vhs demo.tape` to generate the GIF
# Install VHS: https://github.com/charmbracelet/vhs

# Set up the terminal
Output demo.gif
Set FontSize 16
Set Width 1200
Set Height 800
Set Theme "Catppuccin Mocha"

# Demo: Share a port
Type "rift share 3000"
Enter
Sleep 2s

# Show the TUI
Sleep 3s

# Simulate incoming connection
Sleep 2s

# Press 'y' to approve
Type "y"
Sleep 2s

# Show traffic flowing
Sleep 5s

# Exit
Type "q"
Sleep 1s
