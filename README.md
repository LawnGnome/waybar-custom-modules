# Waybar custom modules

This repo contains four custom module providers for Waybar. Each provider is a
binary that you can configure Waybar to use with its custom module support, and
you'll get something useful. Most notably, you can also generate sparklines
with the CPU and memory related modules.

![Custom modules in action](./waybar.png)

The available modules are:

* `cpu`: a clone of Waybar's built in CPU module, except with sparkline support.
  support.
* `cpufreq`: a module to render the current average CPU frequency.
* `mem`: a clone of Waybar's built in memory module, except with sparkline
  support.
* `webcam`: a module that detects if a webcam is attached and displays an
  appropriate icon.

## Platform support

These modules will only work on Linux at present.

The [cpufreq](#cpufreq) module also requires cpufreq directories to be
available in sysfs. (They probably are if you're on an Intel or AMD processor
and a recent kernel, but YMMV, and I'm certainly not an expert on this.)

The [webcam](#webcam) module requires libudev.

## Building

This is a fairly standard Rust project: `cargo build` will give you binaries in
`target/debug`, and `cargo build --release` will give you binaries in
`target/release`.

## Common options

Except for `webcam`, each binary accepts the same general options:

* `--sparkline N`: if `N` is greater than 1, then the output will be formatted
  for use with the [Sparks font](https://github.com/aftertheflood/sparks),
  which [will need to be installed and configured separately](#sparklines).
* `--class CLASS`: to include a specific class for Waybar styling purposes.
* `--interval INTERVAL`: the interval between updates. By default this is 1
  second, but you may want to make this slower in real world use. Any format
  accepted by [humantime](https://github.com/tailhook/humantime) is supported:
  for example, `5s` for 5 seconds.

## Configuring waybar

OK, so how do we get the sparklines going?

1. Install the [Sparks font](https://github.com/aftertheflood/sparks). The
   easiest way to do this is to copy the [OTF
   files](https://github.com/aftertheflood/sparks/tree/master/output/otf) into
   `$HOME/.local/share/fonts`.
2. [Build the binaries in this repository](#building) with
   `cargo build --release`.
3. Create module(s) in the waybar configuration in
   `$HOME/.config/waybar/config`. Here's a snippet of how I have them
   configured (you'll need to replace `$WCM_PATH` with the actual path to your
   `target/release` directory containing the binaries):

    ```json
    "custom/cpu": {
        "format": "{} ",
        "exec": "$WCM_PATH/cpu -i 5s",
        "return-type": "json"
    },
    "custom/cpufreq": {
        "format": "{} ",
        "exec": "$WCM_PATH/cpufreq -i 5s",
        "return-type": "json"
    },
    "custom/mem": {
        "format": "{} ",
        "exec": "$WCM_PATH/mem -i 5s",
        "return-type": "json"
    }
    ```
4. Configure the styles for the new modules. You _must_ set the `font-family`
   for the rendering to work; you'll probably also want to copy the existing
   colour scheme that you have for the default `#cpu` and `#memory` modules.
   Mine looks like this:

    ```css
    #custom-cpufreq,
    #custom-cpu,
    #custom-mem {
        font-family: "Sparks Dot-line Thick";
    }
    ```
5. Restart `waybar` and hope for the best.

## webcam

The webcam module is simpler than the others: it uses udev to monitor if a
video device is attached. Useful if your USB hub is a little flaky first thing
in the morning, you don't always have a video device available even when you
think it's plugged in, and you're often cutting it _real_ fine to make that
Zoom sync.

The defaults should be reasonable, but it does take a handful of options:
`cargo run --bin webcam -- --help` will give you the set.

To configure it into Waybar, something like this should do:

```json
    "custom/webcam": {
        "format": "{}",
        "exec": "$WCM_PATH/webcam",
        "return-type": "json"
    }
```

The default output uses Font Awesome icons. You _probably_ have this configured
already, since Waybar's defaults use it, but if not you'll want something like
this for that module:

```css
#custom-webcam {
    font-family: "Font Awesome 5 Free", sans-serif;
}

#custom-webcam.not-found {
    opacity: 0.2;
}
```

Replace `sans-serif` with whatever font you use elsewhere on Waybar.
