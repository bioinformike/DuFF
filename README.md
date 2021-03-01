#  DuFF [Duplicate File Finder] 

## About 
<img align="right" width="112" height="200" src="https://github.com/bioinformike/DuFF/blob/main/misc/duffman.png">DuFF [**Du**plicate **F**ile **F**inder] is a 
small program written in Rust to find duplicate files in specified directories on a file system in parallel.
<br />
DuFF features:
- Size filtering [min, max, or both!]
- Extension filtering
- Parallel processing

## Table of Contents
<!-- ⛔️ MD-MAGIC-EXAMPLE:START (TOC:collapse=true&collapseText=Click to expand) -->
<details>
<summary>Click to expand</summary>
- [About](#about)
- [Install](#install)
- [Usage](#usage)
  * [API](#api)
  * [Configuration Options](#configuration-options)
- [CLI Usage](#cli-usage)
</details>
<!-- ⛔️ MD-MAGIC-EXAMPLE:END -->

I had originally been implementing all of this in bash, but since I wanted to learn Rust, this seemed ike a good program 
to use to learn it!

Left to implement:
- [ ] Archive functionality [-a flag] (saving hashes for future re-use) 
- [ ] Ability to read in previously computed hashes [-hash arg]
- [ ] Debug functionality (saves working file - doesn't delete) [-d flag]
- [ ] Rename debug functionality 
- [ ] Resume functionality (skip file size examination using user provided previous working file)
  <br /> ```Resume should allow multiple re-entry points depending on if user wants to search dirs again or just skip 
  to hashing... needs more thought.```
- [ ] Saving working file if requested (file with file size data)
- [ ] Need to deal with issues when we traverse into same directory twice.
  <br /> ```Could definitely filter baged on path in file_res, if a path isn't unique delete all but 1 FileResult 
         instance for this path.```
- [ ] Verify DuFF pasts all tests mentioned in this rmlint blog post: https://rmlint.readthedocs.io/en/latest/cautions.html
- [ ] Need to add output and logging.

- [ ] Add low memory mode where instead of building the tree in memory as we recursively search the dir structure in parallel we write our finds out to file. Then use that file for figuring out duplicates, never putting the entire structure in memory.
Tests to write:
- [ ] Add tests for extension filtering 
- [ ] Add tests for size filtering
- [ ] Test Windows compatibility
- [ ] Verify we handle all I/O errors appropriately


Completed
- [x] Include GH issues in program help.
- [x] Program is not ending after walking dir structure - likely has to do with error matching logic in run fn.
- [x] Running program on / where we will get permission denied (good) we are not getting all directories we should.
  <br />
  ```
  # Switching from Walkstate::Quit to Walkstate::Continue fixed this.
  # Another possibility is to use Walkstate::Skip which will not descend into a directory for which it gets an error
  #    (permission denied) but also won't just quit like Walkstate::Quit does.
  # More docs here: https://docs.rs/ignore/0.4.17/ignore/enum.WalkState.html
   ```
- [x] Get File structs inside Config struct
  <br />
  ```
  # Gave up as it doesn't seem possible. Stuffing strings in Config struct and just creating files in main :(
   ```
- [x] Weed out directories (don't need these to be reported from ignore)
- [x] Copy over configuration printing from bash script
  <br />
  ```
  # Currently not coded to wrap the output, so not sure how that will act in a real terminal.
   ```
- [x] Switch all errors to print to stderr
- [x] Add extension limiting functionality --ext
- [x] Add size limiting functionality --size
- [x] Replace size limiting with --ul --ll for upper limit and lower limit and support both at the same time.
- [x] It would be nice if printing --size out was pretty (converted into best size for printing)
- [x] Collect all FileResult structs into one Vector
  <br />```Being stored in file_res```
- [x] Fn: Finding dupes in Vector<FileResult> to send to hashing fn
  <br />
  ```
  # Switched over to using a hashmap of vectors of FileResult
   ```
- [x] Fn: Hashing
- [x] Add mtime, ctime, and atime to FileResult so we can check to see if file changed and whether we have to hash it
     again.
  <br /> ```Added mtime to FileResult, the others probably aren't needed.```
- [x] Refactor code (Move most code out of main.rs)
- [x] Deal with cargo warnings
  <br /> ```Still some warnings left but all for unused variables that I will in future code.```

- [x] Progress bar functionality
  <br /> ```Implemented, -p flag required to show progress ```



