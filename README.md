# SBI (Rust)
Sbi is a linux-first TUI utility for starbound that allows you to combine a set of executables, steam workshop collections, and storage locations as instances.

# Warning
Sbi is built with unix paths and environments in mind and therefore is not guaranteed to work on Windows now or in the future.  
Sbi is in a very experimental state, so stability is not guaranteed and the folder, path, or json structures can change on any commit.  
Many features are simple and non-communicative, so actions such as updating a collection will appear to do nothing but are silently working in the background. (Edit: updating mods now blocks input and shows a spinner!)

# Installing
TODO - binary installation  
SBI relies on `steamcmd` for downloading mods from the workshop, without it, updating a collection will do nothing.
Until an official method for installing is put together, you will just have to rely on the good 'ol `git clone -b rust https://github.com/jaquobia/sbi.git` 
and `cargo install --path ./sbi`. Make sure `$CARGO_HOME/bin` is in your PATH.

# Setup
By default, selecting the `Run Client (Steam)` launch option for an instance will just launch the vanilla game in the steam storage directory with the vanilla executable,
this is why sbi has an alternative launch mode: `sbi -q -- %command%`.
Put this command into your Steam Launch Options for Starbound and it will launch with the intended executable and storage location, or fallback to
the vanilla launch with `%command%` if launched from the Steam play button (game NOT launched from sbi).

Also by default, sbi will have no idea where executables and assets are (To be implemented).
Sbi expects a "vanilla" executable to be defined as it is the default executable name.
Executable and asset definitions will need to be created in `$XDG_DATA_HOME/sbi/config.json`:  
```json
{
  "executables": {
    "vanilla": {
      "bin": "path/to/starbound"
    }
  },
  "vanilla_assets": "path/to/vanilla/assets/containg/dir"
}
```
If an executable has a required assets pak/folder other than the vanilla assets, then add `"custom_assets": "path/to/asset/containing/dir/relative/to/bin"`.  
If an executable has a required library such as steam_api and it is NOT beside the executable, then add `"ld_path": "path/to/ld/containing/dir/relative/to/bin"`.
(All required libraries are expected to be in a single folder, this might change in the future as well as the required relative paths)  
Example:
```json
{
  "executables": {
    "vanilla": {
      "bin": "/home/USER/.local/share/Steam/steamapps/common/Starbound/linux/starbound"
    },
    "xsb2": {
      "bin": "/home/USER/.local/share/Steam/steamapps/common/Starbound/xsb-linux/xclient",
      "custom_assets": "../../xsb-assets"
    }
  },
  "vanilla_assets": "/home/USER/.local/share/Steam/steamapps/common/Starbound/assets"
}
```

# Running
Run `sbi` in the terminal to open the TUI where you can create, modify, and run instances. The home menu has a few keybinds listed at the bottom, other menus should be intiutive enough to get by for now.  
Setting a workshop collection ID in an instance will NOT actively update a collection, the action to download any workshop mods must be initiated through the `(Re)Install Collection` selection in the modify instance menu.

