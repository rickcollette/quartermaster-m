# ATASCII Art, Terminal, and BBS Guidance

## Canvas

Design for a 40-column grid. Graphics 0 is normally 40×24, though BBS software may
reserve rows for status and input. Keep critical art inside 38 columns when it may
be wrapped or bordered.

## Art construction

ATASCII provides lines, corners, diagonals, wedges, blocks, arrows, card suits,
symbols, letters, numbers, and inverse forms. Preserve raw bytes: Unicode is only
an approximation and cannot represent all glyphs or inverse state losslessly.

## Stream rendering

An ATASCII graphics file may mix visible bytes with cursor and editing commands.
A renderer needs a screen buffer, cursor position, tabs, clear-screen, line and
character insertion/deletion, and `$9B` line endings. Cursor movement can overwrite
cells and is the basis of classic ATASCII “break movie” animation.

## BBS detection

ASCII terminals commonly send Return as `$0D` or CR/LF. Atari terminals send
`$9B`. Historic BBS software could use this difference to identify ATASCII users.

## Telnet

Telnet reserves `$FF` as IAC, while ATASCII `$FF` means insert character. Literal
`$FF` sent through Telnet must be doubled as `$FF $FF`, and negotiation commands
must be parsed. Raw TCP does not impose that rule.
