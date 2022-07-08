#![allow(dead_code)]

use ncurses::*;
use core::panic;
use std::process::exit;
use std::io::prelude::*;
use std::path::Path;
use std::fs::File;

enum BranchType {
    Trunk,
    ShootLeft,
    ShootRight,
    Dying,
    Dead
}

struct Config {
    live: i32,
    infinite: i32,
    screensaver: i32,
    print_tree: i32,
    verbosity: i32,
    life_start: i32,
    multiplier: i32,
    base_type: i32,
    seed: i32,
    leaves_size: i32,
    save: bool,
    load: bool,
    target_branch_count: i32,

    time_wait: f32,
    time_step: f32,

    message: String,
    leaves: [char; 64],
    save_file: String,
    load_file: String
}

struct NcursesObjects {
    base_win: WINDOW,
    tree_win: WINDOW,
    message_border_win: WINDOW,
    message_win: WINDOW,

    base_panel: PANEL,
    tree_panel: PANEL,
    message_border_panel: PANEL,
    message_panel: PANEL
}

struct Counters {
    branches: i32,
    shoots: i32,
    shoot_counter: i32
}

#[allow(unused_variables)]
fn quit(conf: &Config, objects: &NcursesObjects, return_code: i32) {
    del_panel(objects.base_panel);
    del_panel(objects.tree_panel);
    del_panel(objects.message_border_panel);
    del_panel(objects.message_panel);

    delwin(objects.base_win);
    delwin(objects.tree_win);
    delwin(objects.message_border_win);
    delwin(objects.message_win);

    // free conf.save_file and conf.load_file

    exit(return_code)
}

fn save_to_file(fname: &str, seed: i32, branch_count: i32) {
    let path = Path::new(fname);

    let mut file = match File::create(&path) {
        Err(e) => panic!("couldn't create save file: {}", e),
        Ok(file) => file,
    };

    match file.write_all(format!("{} {}", &seed.to_string(), &branch_count.to_string()).as_bytes()) {
        Err(e) => panic!("couldn't write to save file: {}", e),
        Ok(_) => println!("wrote to save file"),
    }
}

fn load_from_file(conf: &mut Config) {
    let path = Path::new(conf.load_file.as_str());

    let mut file = match File::open(path) {
        Err(e) => panic!("couldn't open load file: {}", e),
        Ok(file) => file,
    };

    let mut load_data = String::new();
    match file.read_to_string(&mut load_data) {
        Err(e) => panic!("couldn't read load file to string: {}", e),
        Ok(_) => println!("read from load file"),
    }

    let load_data: Vec<i32> = load_data.split_ascii_whitespace().map(|s| s.parse().unwrap()).collect();
    conf.seed = load_data[0];
    conf.target_branch_count = load_data[1];
}

fn finish(conf: &Config, my_counters: &Counters) {
    clear();
    refresh();
    endwin();

    if conf.save {
        save_to_file(&conf.save_file, conf.seed, my_counters.branches);
    }
}

fn main() {
    let mut conf = Config {
        live: 0,
        infinite: 0,
        screensaver: 0,
        print_tree: 0,
        verbosity: 0,
        life_start: 32,
        multiplier: 5,
        base_type: 1,
        seed: 0,
        leaves_size: 0,
        save: true,
        load: true,
        target_branch_count: 0,

        time_wait: 4.0,
        time_step: 0.03,

        message: String::new(),
        leaves: ['\0'; 64],
        save_file: String::from("test_save_file"),
        load_file: String::from("test_save_file"),
    };

    let my_counters = Counters {
        branches: 7,
        shoots: 10,
        shoot_counter: 15,
    };

    if conf.load {
        load_from_file(&mut conf);
    }

    initscr();
    addstr("Hello, World!");
    getch();
    finish(&conf, &my_counters)
    // endwin();

    // save_to_file(&conf.save_file, 5, 10);
}
