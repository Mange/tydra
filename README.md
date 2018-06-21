# tydra

> Terminal Hydra

Customizable shortcut program for your terminal.

**Note:** This readme is currently a specification of the intended use of the
program. It has not yet been implemented!

## Basic idea

The idea is that when running this program you get presented with a menu of
shortcuts. A shortcut can lead to another page of shortcuts and/or run a
command.

The menu will be hidden as long as the command is running, and if the command
is not supposed to return to a page then tydra will quit.

## Use

Define your action file and run tydra with it as your first argument.

```yaml
# Global settings are applied to each page. Each page can override them
# individually if they so wish.
global:
  layout: columns # Render entries in columns
  color: red # Show shortcut letter in red
pages:
  # tydra always start on the "root" page by default:
  root:
    title: Welcome
    groups:
      - title: Web
        entries:
          - shortcut: g
            title: Google
            command: "xdg-open https://www.google.com"
          - shortcut: G
            title: Github
            command: "xdg-open https://www.github.com"
          - shortcut: l
            title: Gitlab
            command: "xdg-open https://www.gitlab.com"

      - title: Desktop
        entries:
          - shortcut: h
            title: Home
            command: "xdg-open ~"
          - shortcut: d
            title: Downloads
            command: "xdg-open ~/Downloads"
          - shortcut: D
            title: Desktop
            command: "xdg-open ~/Desktop"

      - title: Misc
        entries:
          - shortcut: "?"
            title: Show tydra help
            command: "tydra --help | less"
            return: root # Return to the root page after the command has finished.
          - shortcut: p
            color: blue
            title: Packages
            # command: /bin/true # Default when no command is given.
            return: packages # Go to the packages page
          - shortcut: q
            title: Quit
            return: quit # Return to "quit" to quit tydra
  packages:
    title: Packages
    header: "Perform package operations. Go back with {{q}}"
    settings:
      layout: list
    groups:
      - entries:
        - shortcut: r
          title: Refresh package repos
          command: "sudo pacman -Sy"
          return: packages
        - shortcut: u
          title: Show packages that can be upgraded
          command: "pacman -Qu"
          return: packages
        - shortcut: U
          title: Install upgrades
          command: "sudo pacman -Su"
          return: packages
      - settings:
          color: blue
        entries:
        - shortcut: q
          title: Go back
          return: root
```

```
tydra /path/to/actions.yml
```
