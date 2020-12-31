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
- [ ] Weed out directories (don't need these to be reported from ignore)
- [ ] Copy over configuration printing from bash script
- [ ] Switch all errors to print to stderr
- [ ] Add extension limiting functionality --ext
- [ ] Add size limiting functionality --size
- [ ] It would be nice if printing --size out was pretty (converted into best size for printing)
- [ ] Collect all FileResult structs into one Vector
- [ ] Fn: Finding dupes in Vector<FileResult> to send to hashing fn
- [ ] Fn: Hashing
- [ ] Add mtime, ctime, and atime to FileResult so we can check to see if file changed and whether we have to hash it
     again.
  
Testing to do:
- [ ] Verify extension filtering is working
- [ ] Test Windows compatibility
- [ ] Verify we handle all I/O errors appropriately