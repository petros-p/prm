# VHS tape script — records a demo of the PRM REPL into demo.gif.
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
Set TypingSpeed 90ms
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
Sleep 1200ms

# First launch against an empty database walks through one-time setup
# (create the network's owner).
Type "Petros Papadopoulos"
Enter
Sleep 1800ms

# Each add-person answers Nickname/Birthday/How-we-met/Location/Notes with
# either real text or a blank Enter to skip (never 's' — that shortcut
# reads like garbage data when it lands on a field like Birthday), then
# uses 's' at the "Add labels?" y/n prompt to skip the remaining questions.
Type "add-person Testy Exampleton"
Enter
Sleep 900ms
Enter
Sleep 500ms
Enter
Sleep 500ms
Type "Called into a radio show"
Enter
Sleep 700ms
Enter
Sleep 500ms
Enter
Sleep 500ms
Type "s"
Enter
Sleep 1500ms

Type "add-person Billy Bob"
Enter
Sleep 900ms
Type "Billy"
Enter
Sleep 700ms
Type "1955-03-14"
Enter
Sleep 700ms
Type "Fixed my transmission"
Enter
Sleep 700ms
Type "Boston"
Enter
Sleep 700ms
Enter
Sleep 500ms
Type "s"
Enter
Sleep 1500ms

Type "add-person Mike Easter"
Enter
Sleep 900ms
Enter
Sleep 500ms
Enter
Sleep 500ms
Type "Introduced by Testy"
Enter
Sleep 700ms
Enter
Sleep 500ms
Enter
Sleep 500ms
Type "s"
Enter
Sleep 1500ms

Type "list"
Enter
Sleep 2200ms

Type "show-person Billy Bob"
Enter
Sleep 2500ms

Type "find Bob"
Enter
Sleep 2000ms

Type "log Testy Exampleton"
Enter
Sleep 1200ms
Type "1"
Enter
Sleep 900ms
Type "The Garage"
Enter
Sleep 900ms
Type "Brake pads, road trip stories"
Enter
Sleep 900ms
Enter
Sleep 1800ms

Type "set-reminder Billy Bob"
Enter
Sleep 900ms
Type "30"
Enter
Sleep 1800ms

Type "stats"
Enter
Sleep 2500ms

Type "quit"
Enter
Sleep 800ms

# A note dropped into the watched inbox folder (synced over from a phone,
# or any other tool the user already has) waits here for the next launch —
# simulates capturing a note without opening PRM at all.
Hide
Type "mkdir -p .data/inbox && echo 'Ran into Mike at the farmers market, he asked about the transmission fix.' > .data/inbox/note.txt"
Enter
Sleep 300ms
Show

# Relaunching shows who to reach out to next, sorted most-urgent-first,
# right before the REPL prompt — Billy Bob has a reminder set but no
# logged interaction yet, so he surfaces immediately as "never contacted" —
# plus the waiting inbox note captured above.
Type "./prm"
Enter
Sleep 2800ms

Type "quit"
Enter
Sleep 1200ms
