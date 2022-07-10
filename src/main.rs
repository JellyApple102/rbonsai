#![allow(dead_code)]

use ncurses::*;
use rand::{thread_rng, Rng};
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
    screensaver: bool,
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
    base_win: Option<WINDOW>,
    tree_win: Option<WINDOW>,
    message_border_win: Option<WINDOW>,
    message_win: Option<WINDOW>,

    base_panel: Option<PANEL>,
    tree_panel: Option<PANEL>,
    message_border_panel: Option<PANEL>,
    message_panel: Option<PANEL>
}

struct Counters {
    branches: i32,
    shoots: i32,
    shoot_counter: i32
}

#[allow(unused_variables)]
fn quit(conf: &Config, objects: &NcursesObjects, return_code: i32) {
    del_panel(objects.base_panel.expect("could not get base_panel"));
    del_panel(objects.tree_panel.expect("could not get tree_panel"));
    del_panel(objects.message_border_panel.expect("could not get message_border_panel"));
    del_panel(objects.message_panel.expect("could not get message_panel"));

    delwin(objects.base_win.expect("could not get base_win"));
    delwin(objects.tree_win.expect("could not get tree_win"));
    delwin(objects.message_border_win.expect("could not get message_border_win"));
    delwin(objects.message_win.expect("could not get message_win"));

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

fn print_help() {
    println!("Usage: rbonsai [option]...");
    println!();
    println!("rbonsai is a beautifully random bonsai tree generator.");
    println!();
    println!("Options:");
    println!("  -l, --live             live mode: show each step of growth");
    println!("  -t, --time=TIME        in live mode, wait TIME secs between");
    println!("                           steps of growth (must be larger than 0) [default: 0.03]");
    println!("  -i, --infinite         infinite mode: keep growing trees");
    println!("  -w, --wait=TIME        in infinite mode, wait TIME secs between each tree");
    println!("                           generation [default: 4.00]");
    println!("  -S, --screensaver      screensaver mode; equivalent to -lie and");
    println!("                           quit on any keypress");
    println!("  -m, --message=STR      attach message next to the tree");
    println!("  -b, --base=INT         acsii-art plant base to use, 0 is none");
    println!("  -c, --leaf=LIST        list of comma-delimited strings randomly chosen");
    println!("                           for leaves");
    println!("  -M, --multiplier=INT   branch multiplier; higher -> more");
    println!("                           branching (0-20) [default = 5]");
    println!("  -L, --life=INT         life; higher -> more growth (0-200) [default: 32]");
    println!("  -p, --print            print tree to terminal when finished");
    println!("  -s, --seed=INT         seed random number generator");
    println!("  -W, --save=FILE        save progress to file [default: $XDG_CACHE_HOME/rbonsai or $HOME/.cache/rbonsai]");
    println!("  -C, --load=FILE        load progress from file [default: $XDG_CACHE_HOME/rbonsai or $HOME/.cache/rbonsai]");
    println!("  -v, --verbose          increase output verbosity");
    println!("  -h, --help             show help");
}

fn draw_base(base_win: WINDOW, base_type: i32) {
    match base_type {
        1 => {
            wattron(base_win, A_BOLD() | COLOR_PAIR(8));
            wprintw(base_win, ":");
            wattron(base_win, COLOR_PAIR(2));
            wprintw(base_win, "___________");
            wattron(base_win, COLOR_PAIR(11));
            wprintw(base_win, "./~~~\\.");
            wattron(base_win, COLOR_PAIR(2));
            wprintw(base_win, "___________");
            wattron(base_win, COLOR_PAIR(8));
            wprintw(base_win, ":");

            mvwprintw(base_win, 1, 0, " \\                           / ");
            mvwprintw(base_win, 2, 0, "  \\_________________________/ ");
            mvwprintw(base_win, 3, 0, "  (_)                     (_)");

            wattr_off(base_win, A_BOLD());
        },
        2 => {
            wattron(base_win, COLOR_PAIR(8));
            wprintw(base_win, "(");
            wattron(base_win, COLOR_PAIR(2));
            wprintw(base_win, "---");
            wattron(base_win, COLOR_PAIR(11));
            wprintw(base_win, "./~~~\\.");
            wattron(base_win, COLOR_PAIR(2));
            wprintw(base_win, "---");
            wattron(base_win, COLOR_PAIR(8));
            wprintw(base_win, ")");

            mvwprintw(base_win, 1, 0, " (           ) ");
            mvwprintw(base_win, 1, 0, "  (_________)  ");
        },
        _ => (),
    }
}

fn draw_wins(base_type: i32, objects: &mut NcursesObjects) {
    let mut base_width = 0;
    let mut base_height = 0;
    let mut rows = 0;
    let mut cols = 0;

    match base_type {
        1 => {
            base_width = 31;
            base_height = 4;
        },
        2=> {
            base_width = 15;
            base_height = 3;
        },
        _ => ()
    }

    getmaxyx(stdscr(), &mut rows, &mut cols);
    let base_origin_y = rows - base_height;
    let base_origin_x = (cols / 2) - (base_width / 2);

    objects.base_win = Some(newwin(base_height, base_width, base_origin_y, base_origin_x));
    objects.tree_win = Some(newwin(rows - base_height, cols, 0, 0));

    if objects.base_panel != None {
        let p = objects.base_panel.expect("could not get base_panel");
        let w = objects.base_win.expect("could not get base_win");
        replace_panel(p, w);
    } else {
        objects.base_panel = Some(new_panel(objects.base_win.expect("could not get base_win")));
    }

    if objects.tree_panel != None {
        let p = objects.tree_panel.expect("could not get tree_panel");
        let w = objects.tree_win.expect("could not get tree_win");
        replace_panel(p, w);
    } else {
        objects.tree_panel = Some(new_panel(objects.tree_win.expect("could not get tree_win")));
    }

    draw_base(objects.base_win.expect("could not get base_win"), base_type);
}

fn roll(dice: &mut i32, m: i32) {
    let mut rng = thread_rng();
    *dice = rng.gen_range(0..m);
}

fn check_key_press(conf: &Config, my_counters: &Counters) -> bool {
    if conf.screensaver && wgetch(stdscr()) != ERR || (wgetch(stdscr()) == 'q' as i32) {
        finish(conf, my_counters);
        return true;
    }
    false
}

fn main() {
    let mut conf = Config {
        live: 0,
        infinite: 0,
        screensaver: false,
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
