# Atari Text and Graphics Modes

ANTIC generates the display from a display list. Different horizontal bands may
use different text or bitmap modes, enabling mixed-mode screens.

| BASIC mode | Type | Typical full-screen geometry | Notes |
|---:|---|---|---|
| 0 | Text | 40×24 characters | 8×8 glyphs; normal/inverse |
| 1 | Text | 20×24 characters | Double width; four colors |
| 2 | Text | 20×12 characters | Double width/height; four colors |
| 3 | Bitmap | 40×24 pixels | Four colors |
| 4 | Bitmap | 80×48 pixels | Two colors |
| 5 | Bitmap | 80×48 pixels | Four colors |
| 6 | Bitmap | 160×96 pixels | Two colors |
| 7 | Bitmap | 160×96 pixels | Four colors |
| 8 | Bitmap | 320×192 pixels | High resolution |

Adding 16 to a BASIC graphics mode requests a full screen without the default
Graphics 0 text window.

Classic ATASCII art normally targets a 40-column Graphics 0 screen. Display List
Interrupts can change colors or character-set pointers during the frame.

Atari color register values encode hue and luminance. Their RGB appearance varies
by NTSC/PAL/SECAM hardware, monitor, capture path, emulator, and palette preset.
