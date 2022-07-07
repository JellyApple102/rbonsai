#![allow(dead_code)]

use ncurses::*;
use std::process::exit;

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
    save: i32,
    load: i32,
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
    branches: u32,
    shoots: u32,
    shoot_counter: u32
}

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

fn main() {
    initscr();

    addstr("Hello, World!");

    getch();

    endwin();
}
