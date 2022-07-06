use ncurses::*;

fn main() {
    initscr();

    addstr("Hello, World!");

    refresh();

    getch();

    endwin();
}
