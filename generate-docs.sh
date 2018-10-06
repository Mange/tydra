#!/usr/bin/env bash
set -e

# Check for dependencies
if ! hash pandoc 2>/dev/null; then
  echo "You need to have pandoc installed!" > /dev/stderr
  exit 1
fi

if ! hash cargo 2>/dev/null; then
  echo "You need to have cargo installed!" > /dev/stderr
  exit 1
fi

man_pages=(tydra.1.md tydra-actions.5.md)
current_version="$(
  grep -oE --max-count=1 "version = \"[0-9.]+\"" Cargo.toml | cut -d " " -f 3 | tr -d '"'
)"

# Lint the documentation a bit

## Check version numbers
lint-version-number() {
  local file="doc/$1"
  local actual_version
  actual_version="$(
    grep -oE --max-count=1 "Version [0-9.]+" "$file" | cut -d " " -f 2
  )"

  if [[ "$current_version" != "$actual_version" ]]; then
    echo "Error: Expected version in $file to be $current_version but was $actual_version" > /dev/stderr
    exit 1
  fi
}

# Check that all options seem to be in there
match-only-options() {
  LC_LANG=C grep -Eo -- "-(-[a-z-]+|[a-z]\\b)" "$@"
}

lint-options-present() {
  local file="doc/tydra.1.md"
  local actual_options
  local expected_options
  actual_options="$(match-only-options "$file" | sort --unique)"
  expected_options="$(cargo run -- --help 2>/dev/null | match-only-options | sort --unique)"

  if [[ "$actual_options" != "$expected_options" ]]; then
    (
      echo "Seems like the options do not match."
      echo "This was documented, but not part of the --help output:"
      comm -13 <(echo "$expected_options") <(echo "$actual_options")
      echo "This was not documented, despite being part of the --help output:"
      comm -23 <(echo "$expected_options") <(echo "$actual_options")
    ) > /dev/stderr
    exit 1
  fi
}

lint-options-present
for file in "${man_pages[@]}"; do
  echo "Linting $file"
  lint-version-number "$file"
done

# Generate man pages
for input_file in "${man_pages[@]}"; do
  output_file="${input_file%.md}"
  if [[ "$input_file" == "$output_file" ]]; then
    echo "ABORTING! Input and output file would be the same! $input_file" > /dev/stderr
    exit 1
  fi
  echo "Generating $output_file"
  pandoc --standalone --to=man "doc/$input_file" > "doc/$output_file"
done
