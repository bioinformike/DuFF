# dupe_finder

This program can be used to find duplicate files on a filesystem.  It uses the 'ignore' crate to recursively walk a
filesystem in parallel.

I had originally been implementing all of this in bash, but since I wanted to learn Rust, this seemed ike a good program 
to use to learn it!

Left to implement:
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
- [ ] Switch all errors to print to stderr
- [x] Add extension limiting functionality --ext
- [x] Add size limiting functionality --size
- [x] It would be nice if printing --size out was pretty (converted into best size for printing)
- [x] Collect all FileResult structs into one Vector
  <br />```Being stored in file_res```
- [ ] Fn: Finding dupes in Vector<FileResult> to send to hashing fn
- [ ] Fn: Hashing
- [ ] Add mtime, ctime, and atime to FileResult so we can check to see if file changed and whether we have to hash it
     again.
- [x] Refactor code (Move most code out of main.rs)
- [x] Deal with cargo warnings
  <br /> ```Still some warnings left but all for unused variables that I will in future code.```
- [ ] Need to deal with issues when we traverse into same directory twice.
  <br /> ```Could definitely filter baged on path in file_res, if a path isn't unique delete all but 1 FileResult 
         instance for this path.```



Testing to do:
- [ ] Add tests for extension filtering ]
- [ ] Add tests for size filtering
- [ ] Test Windows compatibility
- [ ] Verify we handle all I/O errors appropriately