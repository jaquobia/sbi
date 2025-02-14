# SBI (Rust)
SBI is a linux-first GUI utility for organizing and launching starbound universes with focus on forked clients (OpenStarbound and XStarbound).  
Future plans include re-introducing workshop profile support, but it has been removed in the transition to a GUI interface while a better system is ironed out. The old system worked by downloading a collection using `steamcmd`, moving the files to the profile folder, then deleting the steamcmd folder so it doesn't re-install the previously moved mods due to its verification process. Moving and deleting files unncessisarily is quite a waste not to mention that `steamcmd` has not been the most stable tool - it would fail randomly while downloading a workshop item.

# Warning
Sbi is built with unix paths and environments in mind and therefore is not guaranteed to work on Windows now or in the future.  
Sbi is in a very experimental state, so stability is not guaranteed and the folder, path, or json structures can change on any commit.  
Many features are simple and non-communicative, so actions such as updating a collection will appear to do nothing but are silently working in the background. (Edit: updating mods now blocks input and shows a spinner!)

# Planned Features
- [ ] Linking the vanilla starbound universe to a profile
- [ ] Reading executables from disk
- [ ] Downloading and Updating executables for oSB and xSB from their respective repositories
- [ ] Workshop and Collection support
- [ ] *Maybe* a custom fork for just vanilla with the ability to disable the workshop.

# Installing
TODO - not ready for installation

# Setup
By default, sbi will no executables installed.  
Executable and asset definitions will need to be created in `$XDG_DATA_HOME/sbi/config.json`:  
\*Insert executable setup here once implemented\*

# Running
SBI can be run from the command line with just `sbi` or through steam by replacing the launch args with `sbi -- %command%`

