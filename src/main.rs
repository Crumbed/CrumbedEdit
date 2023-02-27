#![allow(nonstandard_style)]
#![allow(unused_imports)]
#![allow(unused_mut)]

mod buffer;
mod operation;

use std::{io::{ Error, Write, stdout, stdin, Stdout }, fs::{write, File}, env::{Args, args}, time::Instant};

use buffer::{Buffer, Motion, Mode};
use termion::{
    self,
    cursor::{Goto, Hide, Show, self}, 
    screen::IntoAlternateScreen,
    raw::{IntoRawMode, RawTerminal}, 
    input::{TermRead, Keys}, color, style::{Invert, NoInvert},
};



const NAME: &str = "CED - Crumbed Edit";
const VERSION: &str = "Version: a0.2";
const AUTHOR: &str = "by, CrumbedDev : Kai Harrelson";
const DESC: &str = "Ced is a NeoVim alternative and is open source";


fn main() -> Result<(), Error>{
    let mut out = stdout()
        .into_alternate_screen()?
        .into_raw_mode()?;
    let mut stdin = stdin().keys();

    let path = args().nth(1);
    let mut buf = match path {
        Some(ref path) => Buffer::from(&path)?, 
        None => Buffer::new()?
    };

    let mut autosave = Instant::now();

    write!(out, "{}{}{}{}{}{}{}{}{}{}", 
        Hide, 
        color::Fg(color::LightBlack), 
        Goto(1, buf.center-4), 
        format!("{:^width$}", NAME, width = buf.size.x),
        Goto(1, buf.center-2), 
        format!("{:^width$}", VERSION, width = buf.size.x),
        Goto(1, buf.center-1), 
        format!("{:^width$}", AUTHOR, width = buf.size.x),
        Goto(1, buf.center), 
        format!("{:^width$}", DESC, width = buf.size.x), )?;

    if buf.update {
        for row in 0..buf.size.y as u16 { printBlankLine(&mut out, row, buf.size.x)?; }
        write!(out, "{}{}", 
            Goto(buf.visualX, buf.center), 
            Show)?;
        buf.writeLines(&mut out)?;
    } else { for row in 0..buf.size.y as u16 { printBlankLine(&mut out, row, 0)?; } }

    out.flush()?;

    while buf.running {
        if buf.update {
            printStatusBar(&mut out, &mut buf)?;
            buf.restoreCursor(&mut out)?;
            out.flush()?; 
            buf.update = false;
        }

        let input = stdin.next();
        if let Some(Ok(key)) = input {
            match key {
                termion::event::Key::Up => buf.moveCursor(&mut out, Motion::Up(1))?, 
                termion::event::Key::Down => buf.moveCursor(&mut out, Motion::Down(1))?, 
                termion::event::Key::Left => buf.moveCursor(&mut out, Motion::Left(1))?, 
                termion::event::Key::Right => buf.moveCursor(&mut out, Motion::Right(1))?, 

                termion::event::Key::Esc => match buf.mode {
                    Mode::Normal => {
                        save(&mut buf)?;
                        buf.running = false;
                    }, 
                    Mode::Insert => buf.setMode(&mut out, Mode::Normal)?, 

                    _=>(), }, 
                termion::event::Key::Backspace => match buf.mode {
                    Mode::Normal => buf.deleteLine(&mut out)?,
                    Mode::Insert => {
                        let (x, y) = (buf.cPos.x, buf.cPos.y); 
                        if x == 0 || buf.lines[y].is_empty() {
                            if y == 0 { continue; }
                            buf.moveCursor(&mut out, Motion::BeginLine)?;
                            let textAfter = getTextAfterCusor(&mut buf);
                            buf.deleteLine(&mut out)?;
                            if y != buf.lines.len() { buf.moveCursor(&mut out, Motion::Up(1))?; }
                            buf.moveCursor(&mut out, Motion::Endline)?;
                            if !textAfter.is_empty() {
                                buf.insertText(&mut out, &textAfter)?;
                                buf.moveCursor(&mut out, Motion::Left(textAfter.len() as i32))?;
                            }
                            continue;
                        }
                        buf.delete(&mut out, Motion::Left(1))?;
                    }, 

                    _=>(), }

                termion::event::Key::Char(c) => match buf.mode {
                    Mode::Normal => match c {
                        'k' => buf.moveCursor(&mut out, Motion::Up(1))?, 
                        'j' => buf.moveCursor(&mut out, Motion::Down(1))?, 
                        'h' => buf.moveCursor(&mut out, Motion::Left(1))?, 
                        'l' => buf.moveCursor(&mut out, Motion::Right(1))?, 

                        'i' => buf.setMode(&mut out, Mode::Insert)?, 
                        'x' => if buf.deleteChar(&mut out)? {}, 
                        '\n' => buf.insertLine(&mut out, "", Motion::Down(1))?, 
                        'o' => {
                            buf.insertLine(&mut out, "", Motion::Down(1))?;
                            buf.setMode(&mut out, Mode::Insert)?;
                        }, 
                        'O' => {
                            buf.insertLine(&mut out, "", Motion::Up(1))?;
                            buf.setMode(&mut out, Mode::Insert)?;
                        }, 

                        'a' => {
                            buf.setMode(&mut out, Mode::Insert)?;
                            buf.moveCursor(&mut out, Motion::Right(1))?;
                        }, 
                        'A' => {
                            buf.setMode(&mut out, Mode::Insert)?;
                            buf.moveCursor(&mut out, Motion::Endline)?;
                        }, 
                        'I' => { 
                            buf.setMode(&mut out, Mode::Insert)?;
                            buf.moveCursor(&mut out, Motion::BeginLine)?;
                        }, 

                        ':' => {
                            buf.setMode(&mut out, Mode::Cmd)?;
                        },

                        _=>(), },
                    Mode::Insert => { 
                        // WORK ON MACROS \\
                        // let opBuf = &mut buf.opBuf; 
                        // for imac in opBuf.clone().imacros.keys() {
                        //     let start = imac.chars().next().unwrap(); 
                        //     if c == start { continue; }
                        //     if opBuf.checkMacro(c) { continue; }
                        //     break;
                        // }

                        match c {
                            '\n' => {
                                let textAfter = getTextAfterCusor(&mut buf);
                                let line = &mut buf.lines[buf.cPos.y];
                                let mut newLine = line.clone();
                                if !(buf.cPos.x >= line.len()) { newLine = String::from(&line[..buf.cPos.x]); }
                                buf.lines[buf.cPos.y] = newLine;
                                buf.insertLine(&mut out, &textAfter, Motion::Down(1))?;
                            }, 
                            '\t' => buf.insertText(&mut out, "    ")?, 

                            _ => {
                                buf.insertChar(&mut out, c)?;
                                buf.moveCursor(&mut out, Motion::Right(1))?; }, 
                        }
                    }

                    _=>(), }, 

                _=>(), 
            }
        }

        if autosave.elapsed().as_secs() >= 300 && save(&mut buf)? { autosave = Instant::now(); }
    }



    Ok(())
}



pub fn printStatusBar(out: &mut Stdout, buf: &mut Buffer) -> Result<(), Error> {
    let fileName = format!("| {} | ", buf.path.split("/").last().unwrap());
    let mode = format!("{:?}", buf.mode);

    write!(out, "{}{}{}{}{}{}{}{}{}{}{}{}", 
        Hide, 
        color::Fg(color::White), 
        Goto(1, buf.size.y as u16), 
        Invert, 
        format!("{:width$}", "", width = buf.size.x), 
        Goto(2, buf.size.y as u16), 
        mode, 
        Goto(mode.len() as u16+3, buf.size.y as u16), 
        format!("({}, {}), {}", buf.cPos.x, buf.cPos.y, buf.visualX), 
        Goto((buf.size.x - fileName.len()) as u16, buf.size.y as u16), 
        fileName, 
        NoInvert, 
    )?;

    Ok(())
}
pub fn printBlankLine(out: &mut Stdout, row: u16, pad: usize) -> Result<(),  Error>{
    write!(out, "{}{}{}{}", 
        Hide, 
        Goto(1, row), 
        color::Fg(color::LightBlack),
        format!("{:width$}", "~", width = pad),
    )?;
                 

    Ok(()) }
pub fn save(buf: &mut Buffer) -> Result<bool, Error> {
    if buf.path.is_empty() { return Ok(false); }
    let mut file = File::create(buf.path.clone())?;
    for line in buf.lines.iter() {
        file.write_all(&line.as_bytes())?;
        file.write(b"\n")?;
    }
    Ok(true)
}
fn getTextAfterCusor(buf: &mut Buffer) -> String {
    let (x, y) = (buf.cPos.x, buf.cPos.y); 
    let line = buf.lines[y].clone();
    if x >= line.len() || line.len() == 0 { return String::from(""); }

    String::from(&line[x..]) }




