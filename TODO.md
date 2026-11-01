## TODO
- Remove MacOS code. We have added too much features which are only possible on windows so we can remove macos to save time and code.
- Change the config structure. Probably will add some way so you can have multiple config files so its a bit more clear.
- > general.yaml - all stuff which doesnt really fall in any of the other categorized files. also default file which always get read. this file should also include the other files which will make them load on startup. People can also add/extend with their own files or rename files this way.
- > animations.yaml - all windwos animations except the overview animations
- > overview.yaml - all the other overview settings with overview animations
- > stack.yaml - all stack related config settings.
- > window_rules.yaml - all the window rules we currently have with added examples of what is possible.
- > window_effects.yaml - focused_window, other_windows, urgent_window,
- > keybinds.yaml
- > workspaces.yaml #all workspace related stuff. dynamic workspaces, layouts per workspace. special workspace. layouts aswell? if so, maybe another name?
- remove overlay tracking since it isnt really useful anymore.
- remove the hide legacy mode for the hide method since using hide will make our fork behave badly/buggy. also remove place_in_corner since we also removed macos support.

