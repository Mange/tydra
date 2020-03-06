% TYDRA(1) | Version 1.0.2
% Magnus Bergmark <magnus.bergmark@gmail.com>
% March 2020

# NAME

tydra -- Shortcut menu-based task runner, inspired by Emacs Hydra

# SYNOPSIS

| **tydra** \[*-e*|*\--ignore-exit-status*\] \[*-p NAME*|*\--page NAME*\] <*ACTION_FILE*>
| **tydra** \[*-p NAME*|*\--page NAME*\] *\--validate* <*ACTION_FILE*>
| **tydra** *\--help*
| **tydra** *\--version*
| **tydra** *\--generate-completions* <*SHELL*>

# DESCRIPTION

Tydra is a menu-based shortcut runner based on the Hydra system in Emacs.

It works by reading an "action file" that defines the full menu. Each menu has
several pages, where one page at a time can be shown. Each page has one or more
entries, each of which has a shortcut key, a command, and a label.

With these building blocks you can build deeply nested menus with a diverse
range of commands, all behind a very simple shortcut system.

Contrast having to remember long commands (in terminal), or long complicated
shortcuts (in GUI), with a single command/shortcut and then having a menu
overlaid with each entry a single keystroke away.

Tydra makes it easier to add new things to your setup without having to come up
with a globally unique shortcut, while still being possible to remember it even
when it is not used often.

Some possible use-cases:

  * Control your media player.
  * Change your screen brightness and volume without RSI.
  * Bookmark programs with specific arguments, or websites.
  * Keep track of commonly used "recipes" and scripts.

## OPTIONS

**-h**, **\--help**

: Prints quick reference of options.

**-e**, **\--ignore-exit-status**

: Do not exit Tydra when a command fails.

**\--validate**

: Instead of running the menu, exit with exit status *0* if the provided menu
file is valid. If it is not valid, all validation errors will be shown on
*stderr* and the program will exit with a non-zero status code.

**\--version**

: Show the version of the process and exit.

**-p** *NAME*, **\--page** *NAME*

: Start on the page with the provided *NAME*. Defaults to *root* if not
specified. Note that **\--validate** will take this into account too.

**\--generate-completions** *SHELL*

: Generate a completion script for the given shell, print it to standard out,
and exit. This will ignore any other options. You can find a list of supported
shells in the **\--help** output.


# SEE ALSO

**tydra-actions(5)**

: Reference for the action file format, including examples.
