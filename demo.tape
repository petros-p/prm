# VHS tape script — records a short demo of the PRM REPL into demo.gif.
# Run locally with `vhs demo.tape` (requires the VHS CLI: https://github.com/charmbracelet/vhs).
# Regenerated automatically by .github/workflows/demo-gif.yml on every push to
# main that touches the CLI source, so this file should track a real,
# reproducible walkthrough rather than anything env-specific.

Output demo.gif

Set Shell "bash"
Set FontSize 16
Set Width 1200
Set Height 650
Set Theme "Dracula"
Set TypingSpeed 60ms
Set Padding 20

Hide
# Set up a scratch copy of the binary in its own directory so the visible
# launch command below can just be `./prm` with no CI-specific flags/paths
# leaking into the recording, then clear the screen so none of this setup
# (or its echoed command lines) is visible once Show resumes.
Type "rm -rf .demo-data && mkdir -p .demo-data && cp target/release/prm .demo-data/prm && cd .demo-data && clear"
Enter
Sleep 500ms
Show

Type "./prm"
Enter
Sleep 800ms

# First launch against an empty database walks through one-time setup
# (create the network's owner) before the REPL prompt appears.
Type "Demo User"
Enter
Sleep 1000ms

Type "add-person Ada Lovelace"
Enter
Sleep 800ms
Type "s"
Enter
Sleep 1000ms

Type "show-person Ada"
Enter
Sleep 1800ms

Type "find Lovelace"
Enter
Sleep 1500ms

Type "stats"
Enter
Sleep 1800ms

Type "quit"
Enter
Sleep 600ms
