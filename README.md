# SBI (Rust)
SBI is a linux-first GUI utility for organizing and launching starbound universes with focus on forked clients (OpenStarbound and XStarbound).  
Future plans include re-introducing workshop profile support, but it has been removed in the transition to a GUI interface while a better system is ironed out. The old system worked by downloading a collection using `steamcmd`, moving the files to the profile folder, then deleting the steamcmd folder so it doesn't re-install the previously moved mods due to its verification process. Moving and deleting files unncessisarily is quite a waste not to mention that `steamcmd` has not been the most stable tool - it would fail randomly while downloading a workshop item.

# Warning
Sbi is built with unix paths and environments in mind and therefore is not guaranteed to work on Windows now or in the future.  
Sbi is in a very experimental state, so stability is not guaranteed and the folder, path, or json structures can change on any commit.  

# Planned Features
- [x] Linking the vanilla starbound universe to a profile
- [x] Determine plausible locations of the vanilla assets from the `%command%` parameter
- [x] Reading executables from disk
- [ ] Downloading and Updating executables for oSB and xSB from their respective repositories
- [ ] Workshop and Collection support
- [ ] *Maybe* a custom fork for just vanilla with the ability to disable the workshop.

# Installing
TODO - not ready for installation

# Setup
By default, sbi will not have or know where executables are installed.  
Executable and asset definitions will need to be created in `$XDG_DATA_HOME/sbi/config.json`:
1. Extract executable archive into its own folder in `$XDG_DATA_HOME/sbi/executables`. You do not have to move custom assets anywhere special.
2. In `$XDG_DATA_HOME/sbi/config.json` under `executables`, make a new json value as follows:
   ```json
   {
       ...
       "executables": {
           "my_executable": {
               "bin": "/my/path/to/executable",
                "assets": "/my/path/to/custom/assets"
           }
           ...
       }
       ...
   }

   ```
The `assets` field can be a full path or a path relative to the parent folder of `bin`.

# Running
SBI can be run from the command line with just `sbi` or through steam by replacing the launch args with `sbi -- %command%`.

# Help, the game is not launching
* Vanilla assets are missing: In order for any game to launch, the vanilla assets location will need to be specified with the environment variable `SBI_VANILLA_ASSETS_DIR`. An alternative would be to copy or link the vanilla assets into the custom assets folder of each executable. This will hopefully be eased in the future by locating vanilla assets relative to `%command%` from a steam launch.
* `libsteam_api.so` is missing: If for some reason `libsteam_api.so` is not located next to the selected executable, add its path to `LD_LIBRARY_PATH`. SBI will automatically add the folder the executable resides in to `LD_LIBRARY_PATH` but will make no further attempt to locate the library.
* Non-Unicode characters in path: This project makes no attempt to recover paths listed that contain non-unicode characters and can (and will) just filter them out or crash. Ensure that important files are in proper unicode paths.
* I checked everything above but it still doesn't seem to launch!: Check that the executable has executable permissions.
