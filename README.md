# Gitmoji Workflow for Alfred

![GitHub release](https://img.shields.io/github/release/techouse/alfred-gitmoji.svg)
![GitHub All Releases](https://img.shields.io/github/downloads/techouse/alfred-gitmoji/total.svg)
![GitHub](https://img.shields.io/github/license/techouse/alfred-gitmoji.svg)


Search for [Gitmojis](https://gitmoji.dev) using [Alfred](https://www.alfredapp.com/).

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

### Note

Kudos to [Quilljou/alfred-gitmoji-workflow](https://github.com/Quilljou/alfred-gitmoji-workflow) for the initial inspiration.

The gitmoji index was built from from [carloscuesta/gitmoji](https://github.com/carloscuesta/gitmoji/blob/master/src/data/gitmojis.json). The displayed emoji images are from [joypixels/emoji-assets](https://github.com/joypixels/emoji-assets/tree/master/png/128).
The lightning fast search is powered by [Algolia](https://www.algolia.com) using the _same_ index as [gimoji.dev](https://gitmoji.dev).
