#![allow(nonstandard_style)]
#![allow(unused_imports)]
#![allow(unused_mut)]

use std::{fs, io::{Write, Error, Stdout, stdin}};

use termion::{terminal_size, cursor::{Goto, Show, self, Hide}, color, input::TermRead};

use crate::{printBlankLine, operation::OperationBuffer, printStatusBar, save};





pub enum Motion {
    Up(i32), 
    Down(i32), 
    Left(i32), 
    Right(i32),
    Endline, 
    BeginLine, 
}

#[derive(Debug)]
#[derive(PartialEq, Eq)]
pub enum Mode {
    Normal,
    Insert,
    Visual,
    Cmd,
}



pub struct Vec2<T> {
    pub x   :   T, 
    pub y   :   T, }
impl<T> Vec2<T> {
    pub fn xy(x: T, y: T) -> Self {
        Self { x, y }
    } }

pub struct Buffer {
    pub running :   bool, 
    pub path    :   String, 
    pub mode    :   Mode, 
    pub size    :   Vec2<usize>, 
    pub center  :   u16, 
    pub lines   :   Vec<String>,
    pub cPos    :   Vec2<usize>, 
    pub visualX :   u16, 
    pub relNums :   Vec<String>, 
    pub update  :   bool,
    pub opBuf   :   OperationBuffer, }

// CONSTRUCTORS & PRIV FUNCS \\
impl Buffer {
    pub fn from(path: &str) -> Result<Self, Error> {
        let running = true;
        let mode = Mode::Normal;

        let lines = fs::read_to_string(path).expect("Unable to read file");
        let path = path.to_string();
        let lines: Vec<String> = lines.lines().map(|it| it.into()).collect();

        let (x, mut y) = terminal_size()?;
        if y % 2 != 0 { y -= 1; } /*else { y -= 3; }*/
        let size = Vec2::<usize>::xy(x as usize, y as usize);
        let center = y >> 1;

        let cPos = Vec2::<usize>::xy(lines[0].len(), 0);
        let visualX = cPos.x as u16 + 6;

        let mut relNums = Vec::new();
        for y in -(center as i32)..=center as i32 {
            let val = format!("{}{:>4} ", color::Fg(color::LightBlack), y.abs());
            relNums.push(val);
        }

        let update = true;

        let opBuf = OperationBuffer::new();

        Ok(Self {
            running, 
            path,
            mode, 
            size, 
            center, 
            lines,
            cPos, 
            visualX, 
            relNums, 
            update, 
            opBuf, 
        }) } 
    pub fn new() -> Result<Self, Error> {
        let running = true;
        let path = String::new();
        let mode = Mode::Normal;

        let lines = vec![String::new()];

        let (x, mut y) = terminal_size()?;
        if y % 2 != 0 { y -= 1; } /*else { y -= 3; }*/
        let size = Vec2::<usize>::xy(x as usize, y as usize);
        let center = y >> 1;

        let cPos = Vec2::<usize>::xy(0, 0);
        let visualX = cPos.x as u16 + 6;

        let mut relNums = Vec::new();
        for y in -(center as i32)..=center as i32 {
            let val = format!("{}{:>4} ", color::Fg(color::LightBlack), y.abs());
            relNums.push(val);
        }

        let update = false;
        let opBuf = OperationBuffer::new();

        Ok(Self {
            running, 
            path, 
            mode, 
            size, 
            center, 
            lines,
            cPos, 
            visualX, 
            relNums, 
            update, 
            opBuf, 
        }) } 




    fn updateX(self: &mut Self, out: &mut Stdout) -> Result<(), Error>{
        self.visualX = self.cPos.x as u16 + 6;
        let lineLen = self.lines[self.cPos.y].len() as u16 + 6;
        if self.visualX > lineLen { self.visualX = lineLen; }
        write!(out, "{}", 
            Goto(self.visualX, self.center)
        )?;
        Ok(())
    }


    fn drawCurrLine(self: &mut Self, out: &mut Stdout) -> Result<(), Error> {
        write!(out, "{}{}{}{}", 
            Goto(1, self.center), 
            color::Fg(color::White), 
            format!("{:<4} ", self.cPos.y+1), 
            format!("{:width$}", self.lines[self.cPos.y], width = self.size.x - 5),
        )?;

        Ok(())
    }
}

// PUB FUNCS \\ 
impl Buffer {
    // ADDING \\
    pub fn insertChar(self: &mut Self, out: &mut Stdout, c: char) -> Result<(), Error> {
        self.update = true; 
        let line = &mut self.lines[self.cPos.y];

        if self.cPos.x >= line.len() {
            line.push(c);
            self.cPos.x = line.len();
            self.drawCurrLine(out)?;
            return Ok(()); }

        line.insert(self.cPos.x, c); 
        self.drawCurrLine(out)?;
        Ok(()) }
    pub fn insertText(self: &mut Self, out: &mut Stdout, text: &str) -> Result<(), Error> {
        self.update = true; 
        let (x, y) = (self.cPos.x, self.cPos.y);
        let line = &mut self.lines[y];

        if x >= line.len() {
            line.push_str(text);
            self.cPos.x = line.len();
            self.drawCurrLine(out)?;
            self.updateX(out)?;
            return Ok(()); }

        line.insert_str(x, text); 
        self.moveCursor(out, Motion::Right(text.len() as i32))?;
        self.drawCurrLine(out)?;
        self.updateX(out)?;
        Ok(()) }
    pub fn insertLine(self: &mut Self, out: &mut Stdout, text: &str, motion: Motion) -> Result<(), Error> {
        let newLine = String::from(text);
        match motion {
            Motion::Up(..) => self.lines.insert(self.cPos.y, newLine), 
            Motion::Down(..) => {
                self.lines.insert(self.cPos.y+1, newLine);
                self.moveCursor(out, Motion::Down(1))?;
            }

            _=>(), 
        }
        self.writeLines(out)?;
        self.moveCursor(out, Motion::BeginLine)?;

        Ok(()) }



    // DELETING \\
    pub fn deleteChar(self: &mut Self, out: &mut Stdout) -> Result<bool, Error> {
        self.update = true; 
        let (x, y) = (self.cPos.x, self.cPos.y);
        if self.lines[y].len() == 0 { return Ok(false); }
        if x >= self.lines[y].len() { return self.delete(out, Motion::Left(1)); }
        let line = &mut self.lines[y];

        line.remove(x);
        self.drawCurrLine(out)?;

        Ok(true) }
    pub fn delete(self: &mut Self, out: &mut Stdout, motion: Motion) -> Result<bool, Error> {
        self.update = true; 
        let y = self.cPos.y;
        if self.lines[y].len() == 0 { return Ok(false); }
        let mut line = self.lines[y].clone();

        self.moveCursor(out, motion)?;
        line.remove(self.cPos.x);
        self.lines[y] = line;
        self.drawCurrLine(out)?;

        Ok(true) }
    pub fn deleteLine(self: &mut Self, out: &mut Stdout) -> Result<(), Error> {
        let y = self.cPos.y;
        if self.lines.len() <= 1 { return Ok(()); }

        self.lines.remove(y);
        if y >= self.lines.len() { self.moveCursor(out, Motion::Up(1))?; }
        self.writeLines(out)?;
        Ok(()) }


    // UPDATING \\
    pub fn writeLines(self: &mut Self, out: &mut Stdout) -> Result<(), Error> {
        write!(out, "{}", Hide)?;
        self.drawCurrLine(out)?;
        let mut offset: i32 = 1;

        while (self.center as i32 + offset) != self.center as i32 {
            let row = (self.center as i32 + offset) as i32;
            let index = self.cPos.y as i32 + offset;

            if index == -1 || (row != self.size.y as i32 && index == self.lines.len() as i32) { printBlankLine(out, row as u16, self.size.x)?; }

            if index >= self.lines.len() as i32 || row >= self.size.y as i32 { offset = -(self.center as i32); continue; }
            if index < 0 || row < 0 { offset += 1; continue; }

            write!(out, "{}{}{}{}", 
                Goto(1, row as u16), 
                self.relNums[row as usize], 
                color::Fg(color::White), 
                format!("{:width$}", self.lines[index as usize], width = self.size.x - 5),
            )?;

            offset += 1;
        }

        self.update = true; 
        Ok(()) }
    pub fn moveCursor(self: &mut Self, out: &mut Stdout, motion: Motion) -> Result<(), Error> {
        match motion {
            Motion::Up(count) => if (self.cPos.y as i32 - count) >= 0 {
                self.cPos.y -= count as usize;
                self.writeLines(out)?;
            },
            Motion::Down(count) => if (self.cPos.y as i32 + count) < self.lines.len() as i32 {
                self.cPos.y += count as usize;
                self.writeLines(out)?;
            },

            Motion::Left(mut count) => if (self.cPos.x as i32 - count) >= 0 {
                if self.visualX == 6 { self.cPos.x = 0; count = 0; }
                if self.visualX != self.cPos.x as u16+6 { self.cPos.x = self.visualX as usize-6; }
                self.cPos.x -= count as usize;
            },
            Motion::Right(count) => if (self.cPos.x + count as usize) <= self.lines[self.cPos.y].len() {
                self.cPos.x += count as usize;
            }, 
            Motion::Endline => { self.cPos.x = 99999; },
            Motion::BeginLine => { self.cPos.x = 0; }, }

        self.updateX(out)?;
        self.update = true;
        Ok(()) }
    pub fn setMode(self: &mut Self, out: &mut Stdout, mode: Mode) -> Result<(), Error> {
        match mode {
            Mode::Normal => {
                self.mode = mode;
                write!(out, "{}", cursor::SteadyBlock)?;
            }, 
            Mode::Insert => {
                self.mode = mode;
                write!(out, "{}", cursor::BlinkingBlock)?;
            }, 
            Mode::Cmd => {
                self.mode = mode;
                let mut cmd = String::new();
                let mut x = 0 as u16;
                printStatusBar(out, self)?;
                let mut stdin = stdin().keys();
                loop {
                    write!(out, "{}{}{}{}{}{}", 
                        Hide, 
                        Goto(1, self.size.y as u16+1), 
                        color::Fg(color::White), 
                        format!(": {:width$}", cmd, width = self.size.x-3), 
                        Goto(x+3, self.size.y as u16+1), 
                        Show, )?;
                    out.flush()?;

                    let input = stdin.next();
                    if let Some(Ok(key)) = input { match key {
                        termion::event::Key::Esc => {
                            self.setMode(out, Mode::Normal)?;
                            return Ok(());
                        }, 
                        termion::event::Key::Char('\n') => break, 
                        termion::event::Key::Char('\t') => {
                            cmd.insert_str(x as usize, "    ");
                            x += 4;
                        }

                        termion::event::Key::Char(c) => {
                            cmd.insert(x as usize, c);
                            x += 1;
                        }

                        termion::event::Key::Backspace => {
                            if cmd.len() == 0 {
                                self.setMode(out, Mode::Normal)?;
                                return Ok(()); }
                            x -= 1;
                            cmd.remove(x as usize);
                        }

                        _=>(), }
                    }
                } // loop end
                self.setMode(out, Mode::Normal)?;
                if cmd.is_empty() { return Ok(()); }
                let args: Vec<&str> = cmd.split(" ").collect();
                match args[0] {
                    "q" => self.running = false, 
                    "wq" => {
                        if !save(self)? {
                            write!(out, "{}{}{}{}", 
                                Hide, 
                                Goto(1, self.size.y as u16+1), 
                                color::Fg(color::Red), 
                                format!("{:width$}", "No file to save to, try :w <path>", width = self.size.x), )?;
                            out.flush()?;
                            return Ok(()); }
                        self.running = false;
                    }, 
                    "w" => {
                        if args.len() >= 2 { self.path = args[1].to_string(); }
                        if !save(self)? {
                            write!(out, "{}{}{}{}", 
                                Hide, 
                                Goto(1, self.size.y as u16+1), 
                                color::Fg(color::Red), 
                                format!("{:width$}", "No file to save to, try :w <path>", width = self.size.x), )?;
                            out.flush()?;
                        }
                    }, 

                    _ => {
                        write!(out, "{}{}{}{}", 
                            Hide, 
                            Goto(1, self.size.y as u16+1), 
                            color::Fg(color::Red), 
                            format!("{:width$}", "Unidentified command", width = self.size.x), )?;
                        out.flush()?;
                    }, 
                }
            }, 

            _=>(), 
        }
        self.update = true;

        Ok(()) }
    pub fn restoreCursor(self: &mut Self, out: &mut Stdout) -> Result<(), Error> {
        write!(out, "{}{}", 
            Goto(self.visualX, self.center), 
            Show)?;
        Ok(()) }
}









































