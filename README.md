#  DuFF [Duplicate File Finder] 

## About 
<img align="right" width="112" height="200" src="https://github.com/bioinformike/DuFF/blob/main/misc/duffman.png">DuFF [**Du**plicate **F**ile **F**inder] is a 
small program written in Rust to find duplicate files in specified directories on a file system in parallel.
<br />
DuFF features:
- Size filtering [min, max, or both!]
- Extension filtering
- Parallel processing


I had originally been implementing all of this in bash, but since I wanted to learn Rust, this seemed ike a good program 
to use to learn it!

## Install




## Work in Progress
```bash
cargo install duff
```

## Usage 
```bash
./duff -d /home/mike/Desktop -o /home/mike/duff_output -j 4
```

### Required Parameters
The only required argument is where we should search for duplicate files.
* -d [--dir]: The directories you want to search for duplicate files as a comma separated list    
           Ex: -d /home/mike/Desktop,/home/rufus

### Optional Parameters
#### Flags
* -a [--archive]: Tells DuFF to save a copy of all calculated hashes to use in a future DuFF run.
* -g [--log]: Saves the DuFF log file which can be used to resume a DuFF run.
* -p [--prog]: Hides progress information
* -s [--silent]: Hide all console output

##### Arguments
* -l [--lowlim]: Only examine files larger than specified value.
* -u [--uplim]: Only examine files smaller than specified value.
* -j [--jobs]: Tell DuFF the number of threads to use (defaults to 1)
* -e [--ext]: Only examine files with the specified extensions, input as comma separated list.
* -o [--out]: The directory where DuFF should store the output files (defaults to current working directory)
* -r [--resume]: Tell DuFF to skip the directory traversal and instead resume prior run using input log file.
* -x [--hash]: Point DuFF to a set of previously calculated hashes for files.  As long as the mtime is the same, DuFF will not re-calculate hashes for the listed files.

### Left to implement
- [ ] Ability to read in previously computed hashes [-hash arg]

- [ ] Resume functionality (skip file size examination using user provided previous working file)
  <br /> ```Resume should allow multiple re-entry points depending on if user wants to search dirs again or just skip 
  to hashing... needs more thought.```

- [ ] Need to deal with issues when we traverse into same directory twice.
  <br /> ```Could definitely filter baged on path in file_res, if a path isn't unique delete all but 1 FileResult 
         instance for this path.```
- [ ] Verify DuFF pasts all tests mentioned in this rmlint blog post: https://rmlint.readthedocs.io/en/latest/cautions.html


### Tests to write:
- [ ] Add tests for extension filtering 
- [ ] Add tests for size filtering
- [ ] Test Windows compatibility
- [ ] Verify we handle all I/O errors appropriately






