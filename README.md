# dupe_finder

This program can be used to find duplicate files on a filesystem.  It uses the 'ignore' crate to recursively walk a
filesystem in parallel.

I had originally been implementing all of this in bash, but since I wanted to learn Rust, this seemed ike a good program 
to use to learn it!

Left to implement:
- [x] Include GH issues in program help.
- [x] Program is not ending after walking dir structure - likely has to do with error matching logic in run fn.
- [ ] Copy over configuration printing from bash scrip
- [ ] Switch all errors to print to stderr
- [ ] Collect all FileResult structs into one Vector
- [ ] Fn: Finding dupes in Vector<FileResult> to send to hashing fn
- [ ] Fn: Hashing
  
Testing to do:
- [ ] Verify extension filtering is working
- [ ] Test Windows compatibility
- [ ] Verify we handle all I/O errors appropriately