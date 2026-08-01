QuarterMaster/M 1.0.34 is available now.

QuarterMaster/M is a desktop editor and disk workbench for Atari 8-bit text,
ATASCII screens, Atari BASIC programs, and ATR disk images. It now has release
builds for:

- macOS, including Intel and Apple Silicon Macs
- Linux x86_64
- Windows x64

Since 1.0.29, the main changes are:

- Cross-platform release builds for macOS, Linux, and Windows
- Universal macOS DMG packaging
- Linux AppImage, DEB, and RPM packages
- Windows updates now use the portable EXE or setup installer instead of MSI
- Help -> Check for Updates filters packages for the current operating system
- SpartaDOS/X-focused ATR handling
- Faster Ctrl+Shift+Delete line deletion
- Saved files now truncate to the rows that actually contain data
- Better ATR save verification
- More automated frontend and Rust test coverage

If you are on 1.0.29 or another older build, download the latest release from
GitHub first:

https://github.com/rickcollette/quartermaster-m/releases/latest

After installing the latest release, future updates should work from inside the
application through Help -> Check for Updates.

The Windows setup installer includes the Microsoft WebView2 Runtime for offline
installation. A portable Windows EXE is also available for systems where WebView2
is already installed.

The release is not code-signed, so Windows SmartScreen or macOS Gatekeeper may
show a warning. Only bypass that warning if you downloaded the files from the
official GitHub release above.

Source code and documentation:

https://github.com/rickcollette/quartermaster-m

Bug reports, suggestions, and test files are welcome:

https://github.com/rickcollette/quartermaster-m/issues

- Rick Collette (megalith)
