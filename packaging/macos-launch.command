#!/bin/bash
# Double-clicked from Finder. Strips the quarantine flag Gatekeeper puts on
# files extracted from a downloaded archive, so the actual "prm" binary
# doesn't hit a second "unidentified developer" block right after the user
# has already approved opening this launcher script once.
cd "$(dirname "$0")"
chmod +x ./prm
xattr -d com.apple.quarantine ./prm 2>/dev/null
./prm
