# Framework Laptop 16 Keyboard Backlight Daemon

A simple daemon to manage / synchronize the backlight of the Framework Laptop 16's keyboard.
It synchronizes the backlight brightness between the main keyboard input module and any extra modules (i.e. numpad / macropad).
Additionally, it also exposes a kernel LED (via the `uleds` kernel module) which allows for integration with upower / desktop environments.

To use, import this repository's NixOS module (using either the `flake.nix` or the `nix/nixos` entrypoint), and then set the following NixOS options:
```nix
services.fw16-kblight-daemon = {
    enable = true;
    # settings.poll_interval_ms = 50;
};
```

For non-NixOS systems, you will have to resort to building / installing the daemon manually - sorry :/

## Configuration Options

These options will be read from the TOML configuration file specified by the `CONFIG_FILE` environment variable (or `/etc/fw16-kblight.toml` if unspecified).
When using the NixOS module, you may specify use the `services.fw16-kblight-daemon.settings` option instead.

### `poll_interval_ms`

The polling interval used to check for backlight brightness changes.

Default: 200ms

### `rgb_set_hsv_value_to_brightness`

Set the HSV value of RGB-capable input modules to the backlight brightness, mimicking the behavior of the default backlight brightness step keys.

Default: `true`

### `expose_via_uleds`

Expose the keyboard's brightness via the uleds kernel module to allow for control using the Linux LED subsystem and its user (like desktop environments).

Default: `true`