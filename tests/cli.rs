mod common;

use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;
use std::path::PathBuf;
use glob::glob;
use std::io::Write;
use std::fs::File;
use std::fs;


// Not currently functional, so not spending time documenting right now.


// Search dir tests
#[test]
fn search_dir_dne() -> Result<(), Box<dyn std::error::Error>> {

    let dir_name: &str = "search_dir_dne";
    let final_test_dir = format!("./tests/duff_test_data/{}", dir_name);

    // Setup test env
    let mut home_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    home_dir.push(final_test_dir);

    let in_str = home_dir.display().to_string().to_owned();
    let out_str = home_dir.display().to_string().to_owned();

    let mut cmd = Command::cargo_bin("duff")?;

    cmd.arg("-d")
      .arg(in_str)
      .arg("-o")
      .arg(out_str);

    cmd.env("exit", "1")
        .assert()
        .failure()
        .stderr(predicate::str::starts_with(
            "There was an error with the specified directory"));

    Ok(())
}


#[test]
// Files are the same as in perfect_match, we are filtering our matching files
// by size, specifying upper limit of 90B, leaving us with just our matching files, so we
// should still have a dupe pair
// Tests size filtering (upper limit)
//  Success: good input files not filtered out and a duplicate is reported
//  Fail:    good input files filtered out and no duplicate is reported.
fn still_match_upper_lim_filtered() -> Result<(), Box<dyn std::error::Error>> {

    let dir_name: &str = "still_match_upper_lim_filtered";
    let good_file_1: &str   = "good_in_1.txt";
    let good_file_2: &str   = "good_in_2.not_txt";
    let bad_file_1: &str    = "bad_in_1.txt";

    let size_filt: &str      = "90B";

    let final_test_dir = format!("./tests/duff_test_data/{}", dir_name);

    // Setup test env
    let mut home_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    home_dir.push(final_test_dir);
    let home_dir_str = home_dir.display().to_string().to_owned();

    let good_input_1 = format!("{}/{}", home_dir_str, good_file_1);
    let good_input_2 = format!("{}/{}", home_dir_str, good_file_2);
    let bad_input_1 = format!("{}/{}", home_dir_str, bad_file_1);

    let good_data = "Same content\nSame extension\nsame file size\nSame start bytes\nSame end bytes\nSame hash";
    let bad_data = "Same content\nSame extension\nsame file size\nNo, this is new\nSame start bytes\nSame end bytes\nSame hash";

    fs::create_dir_all(home_dir.display().to_string().to_owned())?;

    let mut file = File::create(good_input_1)?;
    write!(&mut file, "{}", good_data)?;
    drop(file);

    file = File::create(good_input_2)?;
    write!(&mut file, "{}", good_data)?;
    drop(file);

    file = File::create(bad_input_1)?;
    write!(&mut file, "{}", bad_data)?;
    drop(file);


    let result_header: String = String::from("File Count\tDuplicate Number");
    let result_row_1: String = format!("1\t1\t{}", good_file_1);
    let result_row_2: String = format!("1\t2\t{}", good_file_2);

    let mut cmd = Command::cargo_bin("duff")?;

    let in_str = home_dir.display().to_string().to_owned();
    let out_str = home_dir.display().to_string().to_owned();

    cmd.arg("-d")
      .arg(in_str)
      .arg("-o")
      .arg(out_str)
      .arg("-u")
      .arg(size_filt);

    cmd.assert()
      .success();

    // Find the report file
    let rep_str = format!("{}/*.report", home_dir_str);

    let mut rep_file = glob(&rep_str)?;
    let curr_file = rep_file.next().unwrap()?;

    let file_content = fs::read_to_string(curr_file)?;

    // Set to false if there is a line in our result file that shouldn't be there
    // meaning not header, row 1, or row 22
    let mut no_line_errors_bool = true;

    let mut header_res = false;
    let mut row_1_res = false;
    let mut row_2_res = false;

    for curr_line in file_content.lines() {
        if curr_line.starts_with(&result_header) {
            header_res = true;
        } else if curr_line.starts_with(&result_row_1) {
            row_1_res = true;
        } else if curr_line.starts_with(&result_row_2) {
            row_2_res = true;
        } else {
            no_line_errors_bool = false;
        }
    }
    // AND all of our result bools together, header_res, row_1_res, and row_2_res
    // get set to true when seen in the result file and no_line_errors_bool only gets
    // set to false if there is an incorrect line in the result file
    let final_bool = header_res & row_1_res & row_2_res & no_line_errors_bool;

    // Clean up before doing assert
    fs::remove_dir_all(home_dir.display().to_string().to_owned())?;

    // Main test, should only succeed if final_bool is true
    assert_eq!(final_bool, true);
    Ok(())
}



#[test]
// Files are the same as in perfect_match, but we are filtering our matching files
// by size, so we shouldn't have any dupe pairs
// Tests size filtering (upper limit)
//  Success: good input files filtered out and no results
//  Fail:    good input files not filtered out and a duplicate is reported.
fn no_dupe_upper_lim_filtered() -> Result<(), Box<dyn std::error::Error>> {

    let dir_name: &str = "no_dupe_upper_lim_filtered";
    let good_file_1: &str   = "good_in_1.txt";
    let good_file_2: &str   = "good_in_2.not_txt";
    let bad_file_1: &str    = "bad_in_1.txt";

    let size_filt: &str      = "70B";


    let final_test_dir = format!("./tests/duff_test_data/{}", dir_name);

    // Setup test env
    let mut home_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    home_dir.push(final_test_dir);
    let home_dir_str = home_dir.display().to_string().to_owned();

    let good_input_1 = format!("{}/{}", home_dir_str, good_file_1);
    let good_input_2 = format!("{}/{}", home_dir_str, good_file_2);
    let bad_input_1 = format!("{}/{}", home_dir_str, bad_file_1);

    let good_data = "Same content\nSame extension\nsame file size\nSame start bytes\nSame end bytes\nSame hash";
    let bad_data = "Same content\nSame extension\nsame file size\nNo, this is new\nSame start bytes\nSame end bytes\nSame hash";

    fs::create_dir_all(home_dir.display().to_string().to_owned())?;


    let mut file = File::create(good_input_1)?;
    write!(&mut file, "{}", good_data)?;
    drop(file);

    file = File::create(good_input_2)?;
    write!(&mut file, "{}", good_data)?;
    drop(file);

    file = File::create(bad_input_1)?;
    write!(&mut file, "{}", bad_data)?;
    drop(file);


    let result_header: String = String::from("File Count\tDuplicate Number");
    let result_row_1: String = format!("1\t1\t{}", good_file_1);
    let result_row_2: String = format!("1\t2\t{}", good_file_2);


    let mut cmd = Command::cargo_bin("duff")?;

    let in_str = home_dir.display().to_string().to_owned();
    let out_str = home_dir.display().to_string().to_owned();

    cmd.arg("-d")
      .arg(in_str)
      .arg("-o")
      .arg(out_str)
      .arg("-u")
      .arg(size_filt);

    cmd.assert()
      .success();

    // Find the report file
    let rep_str = format!("{}/*.report", home_dir_str);

    let mut rep_file = glob(&rep_str)?;
    let curr_file = rep_file.next().unwrap()?;

    let file_content = fs::read_to_string(curr_file)?;

    // Set to false if there is a line in our result file that shouldn't be there
    // meaning not header, row 1, or row 22
    let mut no_line_errors_bool = true;

    let mut header_res = false;
    let mut row_1_res = false;
    let mut row_2_res = false;

    for curr_line in file_content.lines() {
        if curr_line.starts_with(&result_header) {
            header_res = true;
        } else if curr_line.starts_with(&result_row_1) {
            row_1_res = true;
        } else if curr_line.starts_with(&result_row_2) {
            row_2_res = true;
        } else {
            no_line_errors_bool = false;
        }
    }
    // AND all of our result bools together, header_res, row_1_res, and row_2_res
    // get set to true when seen in the result file and no_line_errors_bool only gets
    // set to false if there is an incorrect line in the result file
    let final_bool = header_res & row_1_res & row_2_res & no_line_errors_bool;

    // Clean up before doing assert
    fs::remove_dir_all(home_dir.display().to_string().to_owned())?;

    // Main test, should only succeed if final_bool is false
    assert_eq!(final_bool, false);
    Ok(())
}



#[test]
// Files are the same as in perfect_match, but the extensions are no longer the same
// and we are filtering for only txt files, so we will miss the one pair of dupe.
// Tests extension filtering
//  Success: 'not_txt' filtered out and no results
//  Fail:    'not_txt' not filtered out and a duplicate is reported.
fn no_match_extension_filtered() -> Result<(), Box<dyn std::error::Error>> {

    let dir_name: &str = "no_match_extension_filtered";
    let good_file_1: &str   = "good_in_1.txt";
    let good_file_2: &str   = "good_in_2.not_txt";
    let bad_file_1: &str    = "bad_in_1.txt";

    let ext_filt: &str      = ".txt";


    let final_test_dir = format!("./tests/duff_test_data/{}", dir_name);

    // Setup test env
    let mut home_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    home_dir.push(final_test_dir);
    let home_dir_str = home_dir.display().to_string().to_owned();

    let good_input_1 = format!("{}/{}", home_dir_str, good_file_1);
    let good_input_2 = format!("{}/{}", home_dir_str, good_file_2);
    let bad_input_1 = format!("{}/{}", home_dir_str, bad_file_1);

    let good_data = "Same content\nSame extension\nsame file size\nSame start bytes\nSame end bytes\nSame hash";
    let bad_data = "Same content\nSame extension\nsame file size\nNo, this is new\nSame start bytes\nSame end bytes\nSame hash";

    fs::create_dir_all(home_dir.display().to_string().to_owned())?;


    let mut file = File::create(good_input_1)?;
    write!(&mut file, "{}", good_data)?;
    drop(file);

    file = File::create(good_input_2)?;
    write!(&mut file, "{}", good_data)?;
    drop(file);

    file = File::create(bad_input_1)?;
    write!(&mut file, "{}", bad_data)?;
    drop(file);


    let result_header: String = String::from("File Count\tDuplicate Number");
    let result_row_1: String = format!("1\t1\t{}", good_file_1);
    let result_row_2: String = format!("1\t2\t{}", good_file_2);

    let mut cmd = Command::cargo_bin("duff")?;

    let in_str = home_dir.display().to_string().to_owned();
    let out_str = home_dir.display().to_string().to_owned();

    cmd.arg("-d")
      .arg(in_str)
      .arg("-o")
      .arg(out_str)
      .arg("-e")
      .arg(ext_filt);

    cmd.assert()
      .success();

    // Find the report file
    let rep_str = format!("{}/*.report", home_dir_str);

    let mut rep_file = glob(&rep_str)?;
    let curr_file = rep_file.next().unwrap()?;

    let file_content = fs::read_to_string(curr_file)?;

    // Set to false if there is a line in our result file that shouldn't be there
    // meaning not header, row 1, or row 22
    let mut no_line_errors_bool = true;

    let mut header_res = false;
    let mut row_1_res = true;
    let mut row_2_res = true;

    for curr_line in file_content.lines() {
        if curr_line.starts_with(&result_header) {
            header_res = true;
        } else if curr_line.starts_with(&result_row_1) {
            row_1_res = false;
        } else if curr_line.starts_with(&result_row_2) {
            row_2_res = false;
        } else {
            no_line_errors_bool = false;
        }
    }
    // AND all of our result bools together, header_res, row_1_res, and row_2_res
    // get set to true when seen in the result file and no_line_errors_bool only gets
    // set to false if there is an incorrect line in the result file
    let final_bool = header_res & row_1_res & row_2_res & no_line_errors_bool;

    // Clean up before doing assert
    fs::remove_dir_all(home_dir.display().to_string().to_owned())?;

    // Main test, should only succeed if final_bool is true
    assert!(final_bool);
    Ok(())
}


#[test]
// Files are the same as in perfect_match, but the extensions are no longer the same
// but we aren't filtering out any of our matching files so we should still have one dupe pair.
// Tests extension filtering
//  Success: Nothing filtered out and 1 dupe pair based on content
//  Fail:    File filtered out and no duplicate is found.
fn still_match_diff_extension() -> Result<(), Box<dyn std::error::Error>> {

    let dir_name: &str = "still_match_diff_extension";
    let good_file_1: &str   = "good_in_1.txt";
    let good_file_2: &str   = "good_in_2.not_txt";
    let bad_file_1: &str    = "bad_in_1.txt";

    let ext_filt: &str      = "*";


    let final_test_dir = format!("./tests/duff_test_data/{}", dir_name);

    // Setup test env
    let mut home_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    home_dir.push(final_test_dir);
    let home_dir_str = home_dir.display().to_string().to_owned();


    let good_input_1 = format!("{}/{}", home_dir_str, good_file_1);
    let good_input_2 = format!("{}/{}", home_dir_str, good_file_2);
    let bad_input_1 = format!("{}/{}", home_dir_str, bad_file_1);

    let good_data = "Same content\nSame extension\nsame file size\nSame start bytes\nSame end bytes\nSame hash";
    let bad_data = "Same content\nSame extension\nsame file size\nNo, this is new\nSame start bytes\nSame end bytes\nSame hash";

    fs::create_dir_all(home_dir.display().to_string().to_owned())?;


    let mut file = File::create(good_input_1)?;
    write!(&mut file, "{}", good_data)?;
    drop(file);

    file = File::create(good_input_2)?;
    write!(&mut file, "{}", good_data)?;
    drop(file);

    file = File::create(bad_input_1)?;
    write!(&mut file, "{}", bad_data)?;
    drop(file);


    let result_header: String = String::from("File Count\tDuplicate Number");
    let result_row_1: String = format!("1\t1\t{}", good_file_1);
    let result_row_2: String = format!("1\t2\t{}", good_file_2);

    let mut cmd = Command::cargo_bin("duff")?;

    let in_str = home_dir.display().to_string().to_owned();
    let out_str = home_dir.display().to_string().to_owned();

    cmd.arg("-d")
      .arg(in_str)
      .arg("-o")
      .arg(out_str)
      .arg("-e")
      .arg(ext_filt);

    cmd.assert()
      .success();

    // Find the report file
    let rep_str = format!("{}/*.report", home_dir_str);

    let mut rep_file = glob(&rep_str)?;
    let curr_file = rep_file.next().unwrap()?;

    let file_content = fs::read_to_string(curr_file)?;

    // Set to false if there is a line in our result file that shouldn't be there
    // meaning not header, row 1, or row 22
    let mut no_line_errors_bool = true;

    let mut header_res = false;
    let mut row_1_res = false;
    let mut row_2_res = false;

    for curr_line in file_content.lines() {
        if curr_line.starts_with(&result_header) {
            header_res = true;
        } else if curr_line.starts_with(&result_row_1) {
            row_1_res = true;
        } else if curr_line.starts_with(&result_row_2) {
            row_2_res = true;
        } else {
            no_line_errors_bool = false;
        }
    }
    // AND all of our result bools together, header_res, row_1_res, and row_2_res
    // get set to true when seen in the result file and no_line_errors_bool only gets
    // set to false if there is an incorrect line in the result file
    let final_bool = header_res & row_1_res & row_2_res & no_line_errors_bool;

    // Clean up before doing assert
    fs::remove_dir_all(home_dir.display().to_string().to_owned())?;

    // Main test, should only succeed if final_bool is true
    assert!(final_bool);
    Ok(())
}




#[test]
// Files are the same as in perfect_match, but the extensions are no longer the same
// This one should identify the 2 good files as dupes
fn still_match_despite_extension() -> Result<(), Box<dyn std::error::Error>> {

    // Setup test env
    let mut home_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    home_dir.push("./tests/duff_test_data/ext_match_good");
    let home_dir_str = home_dir.display().to_string().to_owned();

    let good_input_1 = format!("{}/good_in_1.txt", home_dir_str);
    let good_input_2 = format!("{}/good_in_2.not_txt", home_dir_str);
    let bad_input_1 = format!("{}/bad_in_1.txt", home_dir_str);

    let good_data = "Same content\nSame extension\nsame file size\nSame start bytes\nSame end bytes\nSame hash";
    let bad_data = "Same content\nSame extension\nsame file size\nNo, this is new\nSame start bytes\nSame end bytes\nSame hash";

    fs::create_dir_all(home_dir.display().to_string().to_owned())?;


    let mut file = File::create(good_input_1)?;
    write!(&mut file, "{}", good_data)?;
    drop(file);

    file = File::create(good_input_2)?;
    write!(&mut file, "{}", good_data)?;
    drop(file);

    file = File::create(bad_input_1)?;
    write!(&mut file, "{}", bad_data)?;
    drop(file);


    static RESULT_HEADER: &str = "File Count\tDuplicate Number";
    static RESULT_ROW_1: &str = "1\t1\tgood_in_1";
    static RESULT_ROW_2: &str = "1\t2\tgood_in_2";

    let mut cmd = Command::cargo_bin("duff")?;

    let in_str = home_dir.display().to_string().to_owned();
    let out_str = home_dir.display().to_string().to_owned();

    cmd.arg("-d")
      .arg(in_str)
      .arg("-o")
      .arg(out_str);

    cmd.assert()
      .success();

    // Find the report file
    let rep_str = format!("{}/*.report", home_dir_str);

    let mut rep_file = glob(&rep_str)?;
    let curr_file = rep_file.next().unwrap()?;

    let file_content = fs::read_to_string(curr_file)?;

    // Set to false if there is a line in our result file that shouldn't be there
    // meaning not header, row 1, or row 22
    let mut no_line_errors_bool = true;

    let mut header_res = false;
    let mut row_1_res = false;
    let mut row_2_res = false;

    for curr_line in file_content.lines() {
        if curr_line.starts_with(RESULT_HEADER) {
            header_res = true;
        } else if curr_line.starts_with(RESULT_ROW_1) {
            row_1_res = true;
        } else if curr_line.starts_with(RESULT_ROW_2) {
            row_2_res = true;
        } else {
            no_line_errors_bool = false;
        }
    }
    // AND all of our result bools together, header_res, row_1_res, and row_2_res
    // get set to true when seen in the result file and no_line_errors_bool only gets
    // set to false if there is an incorrect line in the result file
    let final_bool = header_res & row_1_res & row_2_res & no_line_errors_bool;

    // Clean up before doing assert
    fs::remove_dir_all(home_dir.display().to_string().to_owned())?;

    // Main test, should only succeed if final_bool is true
    assert!(final_bool);
    Ok(())
}

#[test]
// Good files are the exact same, even extensions are the same, 1 bad file
// has different content and shouldn't show up.
fn perfect_match() -> Result<(), Box<dyn std::error::Error>> {

    // Setup test env
    let mut home_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    home_dir.push("./tests/duff_test_data/perfect_match");
    let home_dir_str = home_dir.display().to_string().to_owned();

    let good_input_1 = format!("{}/perf_match_good_in_1.txt", home_dir_str);
    let good_input_2 = format!("{}/perf_match_good_in_2.txt", home_dir_str);
    let bad_input_1 = format!("{}/perf_match_bad_in_1.txt", home_dir_str);

    let good_data = "Same content\nSame extension\nsame file size\nSame start bytes\nSame end bytes\nSame hash";
    let bad_data = "Same content\nSame extension\nsame file size\nNo, this is new\nSame start bytes\nSame end bytes\nSame hash";

    fs::create_dir_all(home_dir.display().to_string().to_owned())?;


    let mut file = File::create(good_input_1)?;
    write!(&mut file, "{}", good_data)?;
    drop(file);

    file = File::create(good_input_2)?;
    write!(&mut file, "{}", good_data)?;
    drop(file);

    file = File::create(bad_input_1)?;
    write!(&mut file, "{}", bad_data)?;
    drop(file);


    static RESULT_HEADER: &str = "File Count\tDuplicate Number";
    static RESULT_ROW_1: &str = "1\t1\tperf_match_good_in_1.txt";
    static RESULT_ROW_2: &str = "1\t2\tperf_match_good_in_2.txt";

    let mut cmd = Command::cargo_bin("duff")?;

    let in_str = home_dir.display().to_string().to_owned();
    let out_str = home_dir.display().to_string().to_owned();

    cmd.arg("-d")
        .arg(in_str)
        .arg("-o")
        .arg(out_str);

    cmd.assert()
        .success();

    // Find the report file
    let rep_str = format!("{}/*.report", home_dir_str);

    let mut rep_file = glob(&rep_str)?;
    let curr_file = rep_file.next().unwrap()?;

    let file_content = fs::read_to_string(curr_file)?;

    // Set to false if there is a line in our result file that shouldn't be there
    // meaning not header, row 1, or row 22
    let mut no_line_errors_bool = true;

    let mut header_res = false;
    let mut row_1_res = false;
    let mut row_2_res = false;

    for curr_line in file_content.lines() {
        if curr_line.starts_with(RESULT_HEADER) {
            header_res = true;
        } else if curr_line.starts_with(RESULT_ROW_1) {
            row_1_res = true;
        } else if curr_line.starts_with(RESULT_ROW_2) {
            row_2_res = true;
        } else {
            no_line_errors_bool = false;
        }
    }
    // AND all of our result bools together, header_res, row_1_res, and row_2_res
    // get set to true when seen in the result file and no_line_errors_bool only gets
    // set to false if there is an incorrect line in the result file
    let final_bool = header_res & row_1_res & row_2_res & no_line_errors_bool;

    // Clean up before doing assert
    fs::remove_dir_all(home_dir.display().to_string().to_owned())?;

    // Main test, should only succeed if final_bool is true
    assert!(final_bool);
    Ok(())
}