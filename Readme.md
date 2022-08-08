# Substack Downloader

A small cli tool that downloads the public posts from a substack newsletter and saves them locally.

## How to use

```sh
substack-dl url save-dir [--overwrite] [--fmt-html] [--fmt-all]
```

#### Arguments

Argument      | What it does
--------------|-----------------------------------------------------------------------------------------------------
`url`         | The substack domain <domain>.substack.com
`save-dir`    | The local directory where posts should be saved. If the directory doesn't exist, it will be created.
`--overwrite` | Automatically overwrite existing directory (be careful).
`--fmt-html`  | Store HTML versions instead of Markdown.
`--fmt-all`   | Store HTML and Markdown versions.

### Example
```sh
substack-dl nosaj.substack.com ~/save_posts_location --overwrite --fmt-all
```

## Todo
  - [x] Download and parse posts.
  - [x] Save posts to disk as markdown files.
  - [ ] Unit tests for fs operations.
  - [ ] Implement non-tmp functionality.
  - [ ] Implement overwrite flags.
  - [ ] Implement HTML save.
  - [ ] Implement format flags.