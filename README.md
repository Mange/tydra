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
  shortcut_color: red # Show shortcut letter in red
pages:
  # tydra always start on the "root" page by default:
  root:
    title: Welcome
    header: This is the default page.
    footer: "You can always quit using {fg=blue Esc}."
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
            return: true # Return to the same page after the command has finished.
          - shortcut: p
            shortcut_color: blue
            title: Packages
            # command: /bin/true # Default when no command is given.
            return: packages # Go to the packages page
          - shortcut: q
            title: Quit
            return: false # This is default when not specified
  packages:
    title: Packages
    header: "Perform package operations."
    settings:
      layout: list
    groups:
      - entries:
        - shortcut: r
          title: Refresh package repos
          command: "sudo pacman -Sy"
          return: true
        - shortcut: u
          title: Show packages that can be upgraded
          command: "pacman -Qu | less -+F"
          return: true
        - shortcut: U
          title: Install upgrades
          command: |
            sudo pacman -Su
            echo "Press enter to continue"
            read -r x
          return: true
      - settings:
          shortcut_color: blue
        entries:
        - shortcut: q
          title: Go back
          return: root
```

```
tydra /path/to/actions.yml
```

## Still TODO

- A way to dynamically generate pages.
  - Run a command that generates JSON or YAML?

- Configure keybindings.
