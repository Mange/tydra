% TYDRA-ACTIONS(5) | Version 0.1.0
% Magnus Bergmark <magnus.bergmark@gmail.com>
% September 2018

# NAME

tydra-actions -- Tydra action file reference.

# SYNOPSIS

Tydra action files are stored in YAML files. It is recommended that you read up
on YAML so you understand the syntax, such as "[Learn YAML in five minutes](https://www.codeproject.com/Articles/1214409/Learn-YAML-in-five-minutes)". 

Particular areas that might give you issues unless you understand them include:

  * Some strings needing quotes, while others do not.
  * Lists of maps and indentation rules for them.

The main outline of the file can be illustrated using this small example:

```yaml
global:
  layout: columns
pages:
  root:
    groups:
      - entries:
          - shortcut: a
            title: Print a
            command: echo a
```

**global** (optional)

: See **GLOBAL SETTINGS** below.

**pages** (required)

: A map of pages, where the name of the page is the key and the value is the
page specification. See **PAGE** below.

## GLOBAL SETTINGS

The global settings allows you to set configuration that applies by default to
pages / groups / entries. You don't need to provide any settings if you don't
want to.

**layout** (optional)

: Sets the default layout for pages that don't override it. Allowed values are
*columns* and *list*.

**shortcut_color** (optional)

: Sets the color to use when rendering the shortcut key. Allowed values are
*reset*, *black*, *blue*, *cyan*, *green*, *magenta*, *red*, *white*, and
*yellow*.

## PAGE

Pages contains groups of entries (see **GROUP**) and some additional settings
and texts. The groups will be rendered in the *layout* set in the settings.

**title** (optional)

: The title of the page.

**header** (optional)

: Introduction text before showing the entries. You could explain the page here.

**footer** (optional)

: Text after showing the entries. You could place notices or some other
information about the page here.

**settings** (optional)

: The same collection of settings that are inside the **GLOBAL SETTINGS**, only
here they only apply to the current page instead. If this is not provided, then
the global settings will be used as-is.

**groups** (required)

: A list of groups. See **GROUPS**.

## GROUPS

Groups is a single grouping of menu entries (see **ENTRY**) along with some
additional metadata and settings.

**title** (optional)

: The title of this group.

**settings** (optional)

: The same settings as **GLOBAL SETTINGS**, but only the settings that affect
entries will be taken into account. Settings not provided here will be
inherited from the parent **page**, then the global settings.

**entries** (required)

: A list of entries that should be inside this group. See **ENTRY**.

## ENTRY

Entries are the thing that you select in menus. They have a lot of things to
customize.

**title** (required)

: The title of the entry. This is the text that you'll see on the screen.

**shortcut** (required)

: A single character that will be used to trigger this entry. For example *a*
would mean that the entry is selected by pressing *a* on your keyboard.
Shortcuts must be unique for a single **page** or else you will get a
validation error.

**command** (optional)

: The command to execute when triggering this entry. It is optional because
sometimes you want entries to navigate to different pages, and in those cases
you do not need a command. See **return**.
Commands can be given in two formats. Either as a single string, which means
that it will be run as a shell-script to */bin/sh*, or as a structure with a
*name* and *args* key.

```yaml
command: xdg-open https://example.net

command: |
  if [[ -f ~/.local/bookmarks.html ]]; then
    xdg-open ~/.local/bookmarks.html
  fi

command:
  name: xdg-open
  args:
    - "https://example.net"
```

**shortcut_color** (optional)

: Sets the color for the shortcut character when shown to the user. Will be
taken from the **settings** in the parent **GROUP**, then **PAGE**, and lastly
the **GLOBAL SETTING** if not provided before then. See **GLOBAL SETTINGS** for
a complete list of supported colors.

**mode** (optional)

: Instructs tydra on how to run the command. Supported values are *normal*,
*wait*, *exec* and *background*.

*normal* (default)

: This mode pauses tydra and runs the **command** in your normal terminal screen.

*wait*

: This is identical to *normal*, but waits for the user to press *enter* before
continuing after the command has finished. This is useful for entries that are
meant to resume tydra after being run (see **return**) where the user want to
see the output before continuing.

*exec*

: This mode replaces tydra with the new command. This is great if you don't
want to resume tydra after the command (see **return**) or if the process is
long-running as *exec* prevents tydra from being the process parent.

*background*

: Runs the command in the background, disconnected from tydra. Depending on
**return** tydra either resumes immediately, or exits immediately. This is
great for spawning GUI applications, or to run commands that do not require
feedback (like increasing volume).

**return** (optional)

: Sets the return mode of the entry. Allowed values are *false*, *true*, or the
name of another page. Note that when (or if at all) tydra returns depend on the
**mode** of the entry; if you *exec* the command tydra cannot return after it.
If you run the command in *background*, then this return action will be taken
immediately, but both *normal* and *wait* will only perform it after the
command finishes. If tydra does not run with the **\--ignore-exit-status**
option, then a failing command will also exit tydra. This is to help you find
bugs in your scripts.

*true*

: Return to the same page again after the command runs.

*false*

: Exit tydra after the command runs.

*Another page's name*

: Return to this page after the command runs. If the page name cannot be found
in the action file, you will get a validation error.

# EXAMPLES

Examples are not currently provided.
