#![allow(dead_code)]

use ncurses::*;
use rand::{thread_rng, Rng};
use core::panic;
use std::process::exit;
use std::io::prelude::*;
use std::path::Path;
use std::fs::File;
use std::thread;
use std::time::Duration;
use std::fmt;

#[derive(PartialEq, Clone, Copy)]
enum BranchType {
    Trunk,
    ShootLeft,
    ShootRight,
    Dying,
    Dead
}

impl BranchType {
    fn from_i32(value: i32) -> BranchType {
        match value {
            0 => BranchType::Trunk,
            1 => BranchType::ShootLeft,
            2 => BranchType::ShootRight,
            3 => BranchType::Dying,
            4 => BranchType::Dead,
            _ => panic!("invalid branch i32 conversion"),
        }
    }
}

impl fmt::Display for BranchType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            BranchType::Trunk => write!(f, "Trunk"),
            BranchType::ShootLeft => write!(f, "ShootLeft"),
            BranchType::ShootRight => write!(f, "ShootRight"),
            BranchType::Dying => write!(f, "Dying"),
            BranchType::Dead => write!(f, "Dead"),
        }
    }
}

struct Config {
    live: bool,
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

fn update_screen(time_step: f32) {
    update_panels();
    doupdate();

    let dur = Duration::from_secs_f32(time_step);
    thread::sleep(dur);
}

fn choose_color(b_type: BranchType, tree_win: WINDOW) {
    let mut rng = thread_rng();

    match b_type {
        BranchType::Trunk | BranchType::ShootLeft | BranchType::ShootRight => {
            if rng.gen_range(0..2) == 0 {
                wattron(tree_win, A_BOLD() | COLOR_PAIR(11));
            } else {
                wattron(tree_win, COLOR_PAIR(3));
            }
        },
        BranchType::Dying => {
            if rng.gen_range(0..10) == 0 {
                wattron(tree_win, A_BOLD() | COLOR_PAIR(2));
            } else {
                wattron(tree_win, COLOR_PAIR(2));
            }
        },
        BranchType::Dead => {
            if rng.gen_range(0..3) == 0 {
                wattron(tree_win, A_BOLD() | COLOR_PAIR(10));
            } else {
                wattron(tree_win, COLOR_PAIR(10));
            }
        }
    }
}

fn set_deltas(b_type: BranchType, life: i32, age: i32, multiplier: i32, return_dx: &mut i32, return_dy: &mut i32) {
    let mut dx: i32 = 0;
    let mut dy: i32 = 0;
    let mut dice: i32 = 0;

    let mut rng = thread_rng();

    match b_type {
        BranchType::Trunk => {
            if age <= 2 || life < 4 {
                dy = 0;
                dx = rng.gen_range(0..3) - 1;
            } else if age < multiplier * 3 {
                if age % (multiplier * 0.5 as i32) == 0 { dy = -1; } else { dy = 0; }

                roll(&mut dice, 10);
                if dice == 0 { dx = -2; }
                else if (1..=3).contains(&dice) { dx = -1; }
                else if (4..=5).contains(&dice) { dx = 0; }
                else if (6..=8).contains(&dice) { dx = 1; }
                else if dice == 9 { dx = 2; }
            } else {
                roll(&mut dice, 10);
                if dice > 2 { dy = -1; }
                else { dy = 0; }
                dx = rng.gen_range(0..3) - 1;
            }
        },
        BranchType::ShootLeft => {
            roll(&mut dice, 10);
            if (0..=1).contains(&dice) { dy = -1; }
            else if (2..=7).contains(&dice) { dy = 0; }
            else if (8..=9).contains(&dice) { dy = 1; }

            roll(&mut dice, 10);
            if (0..=1).contains(&dice) { dx = -2; }
            else if (2..=5).contains(&dice) { dx = -1; }
            else if (6..=8).contains(&dice) { dx = 0; }
            else if dice == 9 { dx = 1; }
        },
        BranchType::ShootRight => {
            roll(&mut dice, 10);
            if (0..=1).contains(&dice) { dy = -1; }
            else if (2..=7).contains(&dice) { dy = 0; }
            else if (8..=9).contains(&dice) { dy = 1; }

            roll(&mut dice, 10);
            if (0..=1).contains(&dice) { dx = 2; }
            else if (2..=5).contains(&dice) { dx = 1; }
            else if (6..=8).contains(&dice) { dx = 0; }
            else if dice == 9 { dx = -1; }
        },
        BranchType::Dying => {
            roll(&mut dice, 10);
            if (0..=1).contains(&dice) { dy = -1; }
            else if (2..=8).contains(&dice) { dy = 0; }
            else if dice == 9 { dy = 1; }

            roll(&mut dice, 15);
            if dice == 0 { dx = -3; }
            else if (1..=2).contains(&dice) { dx = -2; }
            else if (3..=5).contains(&dice) { dx = -1; }
            else if (6..=8).contains(&dice) { dx = 0; }
            else if (9..=11).contains(&dice) { dx = 1; }
            else if (12..=13).contains(&dice) { dx = 2; }
            else if dice == 14 { dx = 3; }
        },
        BranchType::Dead => {
            roll(&mut dice, 10);
            if (0..=2).contains(&dice) { dy = -1; }
            else if (3..=6).contains(&dice) { dy = 0; }
            else if (7..=9).contains(&dice) { dy = 1; }
            dx = rng.gen_range(0..3) - 1;
        }
    }

    *return_dx = dx;
    *return_dy = dy;
}

fn choose_string(conf: &Config, mut b_type: BranchType, life: i32, dx: i32, dy: i32) -> String {
    const MAX_STR_LEN: usize = 32;
    let mut branch_str: String = String::with_capacity(MAX_STR_LEN);

    branch_str.push('?');
    if life < 4 { b_type = BranchType::Dying };

    match b_type {
        BranchType::Trunk => {
            if dy == 0 { branch_str = "/~".to_string(); }
            else if dx < 0 { branch_str = "\\|".to_string(); }
            else if dx == 0 { branch_str = "/|\\".to_string(); }
            else if dx > 0 { branch_str = "|/".to_string(); }
        },
        BranchType::ShootLeft => {
            if dy > 0 { branch_str = "\\".to_string(); }
            else if dy == 0 { branch_str = "\\_".to_string(); }
            else if dx < 0 { branch_str = "\\|".to_string(); }
            else if dx == 0 { branch_str = "/|".to_string(); }
            else if dx > 0 { branch_str = "/".to_string(); }
        },
        BranchType::ShootRight => {
            if dy > 0 { branch_str = "/".to_string(); }
            else if dy == 0 { branch_str = "_/".to_string(); }
            else if dx < 0 { branch_str = "\\|".to_string(); }
            else if dx == 0 { branch_str = "/|".to_string(); }
            else if dx > 0 { branch_str = "/".to_string(); }
        },
        BranchType::Dying | BranchType::Dead => {
            let mut rng = thread_rng();

            branch_str.clear();
            let i: i32 = rng.gen_range(0..conf.leaves_size); // does this emulate the og?
            let c: char = conf.leaves[i as usize];

            for _ in 0..MAX_STR_LEN {
                branch_str.push(c);
            }
        }
    }

    branch_str
}

#[allow(unused_assignments)] // 'age is assigned but not used' warning
fn branch(conf: &Config, objects: &NcursesObjects, my_counters: &mut Counters, mut y: i32, mut x: i32, b_type: BranchType, mut life: i32) {
    my_counters.branches += 1;
    let mut dx: i32 = 0;
    let mut dy: i32 = 0;
    let mut age: i32 = 0;
    let mut shoot_cooldown: i32 = conf.multiplier;

    let mut rng = thread_rng();

    while life > 0 {
        if check_key_press(conf, my_counters) {
            quit(conf, objects, 0);
        }

        life -= 1;
        age = conf.life_start - life;

        set_deltas(b_type, life, age, conf.multiplier, &mut dx, &mut dy);

        let max_y: i32 = getmaxy(objects.tree_win.unwrap());
        if dy > 0 && y > (max_y - 2) { dy -= 1; }

        if life < 3 {
            branch(conf, objects, my_counters, y, x, BranchType::Dead, life)
        } else if (b_type == BranchType::Trunk || b_type == BranchType::ShootLeft || b_type == BranchType::ShootRight) && life < (conf.multiplier + 2) {
            branch(conf, objects, my_counters, y, x, BranchType::Dying, life);
        } else if (b_type == BranchType::Trunk && rng.gen_range(0..3) == 0) || (life % conf.multiplier == 0) {
            if rng.gen_range(0..8) == 0 && life > 7 {
                shoot_cooldown = conf.multiplier * 2;
                branch(conf, objects, my_counters, y, x, BranchType::Trunk, life + rng.gen_range(0..5) - 2);
            } else if shoot_cooldown <= 0 {
                shoot_cooldown = conf.multiplier * 2;

                let shoot_life: i32 = life + conf.multiplier;

                my_counters.shoots += 1;
                my_counters.shoot_counter += 1;
                if conf.verbosity > 0 {
                    mvwprintw(objects.tree_win.unwrap(), 4, 5, format!("shoots: {}", my_counters.shoots).as_str());
                }

                branch(conf, objects, my_counters, y, x, BranchType::from_i32((my_counters.shoot_counter % 2) + 1), shoot_life);
            }
        }
        shoot_cooldown -= 1;

        if conf.verbosity > 0 {
            mvwprintw(objects.tree_win.unwrap(), 5, 5, format!("dx: {}", dx).as_str());
            mvwprintw(objects.tree_win.unwrap(), 6, 5, format!("dy: {}", dy).as_str());
            mvwprintw(objects.tree_win.unwrap(), 7, 5, format!("type: {}", b_type).as_str());
            mvwprintw(objects.tree_win.unwrap(), 8, 5, format!("dx: {}", dx).as_str());
        }

        x += dx;
        y += dy;

        choose_color(b_type, objects.tree_win.unwrap());

        let branch_str: String = choose_string(conf, b_type, life, dx, dy);

        // i do not think i need to do anything with wide characters,
        // i think rust handles unicode stuff better by default than C
        //
        // i could be (probably am) wrong but thats a problem for another time

        mvwprintw(objects.tree_win.unwrap(), y, x, branch_str.as_str());

        wattroff(objects.tree_win.unwrap(), A_BOLD());

        if conf.live && !(conf.load && my_counters.branches < conf.target_branch_count) {
            update_screen(conf.time_step);
        }
    }
}

fn add_spaces(message_win: WINDOW, count: i32, line_position: &mut i32, max_width: i32) {
    if *line_position < (max_width - count) {
        for _ in 0..count {
            wprintw(message_win, " ");
            *line_position += 1;
        }
    }
}

fn main() {
    let mut conf = Config {
        live: false,
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
