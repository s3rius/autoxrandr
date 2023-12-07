# Autoxrandr

Autoxrandr is a simple program that tracks your layout and automatically applies it when you connect or disconnect a monitor.

## Installation

```bash
git clone https://github.com/s3rius/autoxrandr
cd autoxrandr
makepkg -fsri
```

These commands will clone the repository, build the package and install it.

## Usage

Ideally, you would want to run autoxrandr on startup. You can do this by adding the following line to your `~/.xinitrc`:

```bash
autoxrandr &
```

Or maybe if you're using a display manager, like i3, you can add it to your `~/.config/i3/config`:

```bash
## 0.5 here to wait for the monitors to be fully connected and ready
exec --no-startup-id sleep 0.5 && autoxrandr
```

## Configuration

Autoxrandr uses only CLI arguments for configuration. You can see the list of available arguments by running `autoxrandr --help`.

### On remap

Autoxrandr might run a specific command when it applies a layout. You can specify this command by using the `--on-remap` argument. For example, if you want to run nitrogen to reapply background, when autoxrandr applies a layout, you can do this:

```bash
autoxrandr --on-remap "nitrogen --restore"
```

Or if you want to run multiple commands, you can do this:

```bash
autoxrandr --on-remap '/bin/sh -c "sleep 0.5 && (killall polybar || echo "poly is dead") && nitrogen --restore && ~/.config/polybar/launch.sh"'
```

This command waits for 0.5 seconds, kills polybar and then reapplies the background and restarts the polybar.

Additionally, the command that runs on remap can be a long-running process. On the next remap, autoxrandr will kill the previous process and will run the new one.

### Background

The autoxrandr can also be configured to run in the background. You can do this by using the `--background` argument. For example:

```bash
autoxrandr --background
```