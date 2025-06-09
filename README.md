# SBI (Rust)
SBI is a linux-first GUI utility for organizing and launching starbound universes with focus on forked clients (OpenStarbound and XStarbound).  
Future plans include re-introducing workshop profile support, but it has been removed in the transition to a GUI interface while a better system is ironed out.  
The old system worked by downloading a collection using `steamcmd`, moving the files to the profile folder, then deleting the steamcmd folder so it doesn't re-install 
the previously moved mods due to its verification process. Moving and deleting files unncessisarily is quite a waste not to mention that `steamcmd` has not been the most stable 
tool - it would fail randomly while downloading a workshop item.

# Warning
Sbi is built with unix paths and environments in mind and therefore is not guaranteed to work on Windows now or in the future.  
Sbi is in a very experimental state, so stability is not guaranteed and the folder or json structures can change on any commit.  

# Planned Features
- [x] Link the vanilla starbound universe to a default profile
- [x] Determine plausible locations of the vanilla assets from the `%command%` parameter
- [x] Reading executables from disk
- [x] Adding executables through settings menu
- [ ] Profile specific settings
- [ ] Workshop and Collection support
- [ ] *Maybe* Downloading and Updating executables for oSB and xSB from their respective repositories
- [ ] *Maybe* a custom fork for just vanilla with the ability to disable the workshop.

# Installing
For now, this can only be installed using the usual cargo syntax. Either use the `cargo install` command or
manually build the artifact then move it to a directory on your PATH.

# Setup
By default, sbi will not have any executables installed.  
It is recommended to download and unzip [OpenStarbound](https://github.com/OpenStarbound/OpenStarbound/releases) or [XStarbound](https://github.com/xStarbound/xStarbound/releases)
into an easily path-able location. 
It is also recommended to run the vanilla game at least once, there is some issue where launching an untouched vanilla profile will cause an error and prevent loading.

# Running
SBI can be run from the command line with just `sbi` or through steam by replacing the launch args with `sbi -- %command%`.  
Note: Steam requires the -- %command% to be present or it will try to use the word 'sbi' as an additional parameter for starbound.  
Note: On sandboxed environments, running sbi through the cli will probably work fine, but oSB and xSB have dynamic dependencies, e.g. xSB requires SDL2 on the lib path. On NixOS,
sbi declares a dependency on SDL2 for this very reason, and the rest of the dependencies (as of June 2025) can solved by steam-run.  
When you run SBI for the first time, you will need to enter the settings menu, fill out the name, pick the binary and optionally an asset folder, 
then press the `Add` buttton in order to create a new executable definition.

# Help, the game is not launching
* Vanilla assets are missing: In order for any game to launch, the vanilla assets are required. SBI makes a good attempt to find the assets in regular places,
however a location can be specified with the environment variable `SBI_VANILLA_ASSETS_DIR`.
* `libsteam_api.so` is missing: If for some reason `libsteam_api.so` is not located next to the selected executable's binary, add its path to `LD_LIBRARY_PATH`. SBI will automatically add the folder the executable resides in to `LD_LIBRARY_PATH` but will make no further attempt to locate the library.
* Non-Unicode characters in path: This project makes no attempt to recover paths listed that contain non-unicode characters and can (and will) just filter them out or crash. Ensure that important files are in proper unicode paths.
* I checked everything above but it still doesn't seem to launch!: Check that the executable has executable permissions. If there are still issues, open an issue with proper logs and starbound version.
