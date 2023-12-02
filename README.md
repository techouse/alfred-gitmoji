# Gitmoji Workflow for Alfred

![GitHub release](https://img.shields.io/github/release/techouse/alfred-gitmoji.svg)
![GitHub All Releases](https://img.shields.io/github/downloads/techouse/alfred-gitmoji/total.svg)
![GitHub](https://img.shields.io/github/license/techouse/alfred-gitmoji.svg)
[![Gitmoji](https://img.shields.io/badge/gitmoji-%20😜%20😍-FFDD67.svg?style=flat)](https://gitmoji.dev)
[![GitHub Sponsors](https://img.shields.io/github/sponsors/techouse)](https://github.com/sponsors/techouse)


Search for [Gitm:stuck_out_tongue_winking_eye:jis](https://gitmoji.dev) using [Alfred](https://www.alfredapp.com/).

![demo](demo.gif)

## Installation

1. [Download the latest version](https://github.com/techouse/alfred-gitmoji/releases/latest)
2. Install the workflow by double-clicking the `.alfredworkflow` file
3. You can add the workflow to a category, then click "Import" to finish importing. You'll now see the workflow listed in the left sidebar of your Workflows preferences pane.

## Usage

Just type `gm` followed by your search query.

```
gm update
```

Either press `⌘Y` to Quick Look the result, or press `<enter>` copy it to your clipboard.

### Modifier keys

- <kbd>return</kbd> (↵): Copy the code of the selected gitmoji (e.g. `:bug:`) directly to your front-most application.
- <kbd>option+return</kbd> (⌥↵): Copy the symbol of the selected emoji) (e.g. "🐛") to your clipboard.
- <kbd>ctrl+return</kbd> (⌃↵): Copy the hexadecimal HTML Entity of the selected emoji) (e.g. `&#x1f41b;`) to your clipboard.
- <kbd>shift+return</kbd> (⇧↵): Copy the Python source of the selected emoji) (e.g. `u"\U0001F41B"`) to your clipboard.
- <kbd>shift+ctrl+return</kbd> (⇧⌃↵): Copy the formal Unicode notation of the selected emoji) (e.g. `U+1F41B`) to your clipboard.
- <kbd>cmd+return</kbd> (⌘↵): Copy the code of the selected emoji (e.g. `:bug:`) to your clipboard.

### Notes

Kudos to [leolabs/alfred-gitmoji](https://github.com/leolabs/alfred-gitmoji) for the initial inspiration.

The gitmoji index was built from [carloscuesta/gitmoji](https://github.com/carloscuesta/gitmoji). The displayed emoji images are from [joypixels/emoji-assets](https://github.com/joypixels/emoji-assets).
The lightning fast search is powered by [Algolia](https://www.algolia.com) using the _same_ index as [gimoji.dev](https://gitmoji.dev).
