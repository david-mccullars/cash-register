use colored::Colorize;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::style::Print;
use crossterm::terminal::{
    self, size as terminal_size, ClearType, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{cursor, ExecutableCommand};
use rusty_money::{iso, Money};
use std::fmt::Display;
use std::io::{stdout, Stdout, Write};
use unicode_truncate::UnicodeTruncateStr;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut stdout = stdout();
    stdout.execute(EnterAlternateScreen)?;
    terminal::enable_raw_mode()?;

    let mut history: Vec<String> = vec![];
    let mut total = Money::from_major(0, iso::USD);
    let mut buffer = String::new();

    loop {
        display(&mut stdout, &buffer, &history)?;

        if let Event::Key(e) = event::read()? {
            match e.code {
                KeyCode::Char('c') if e.modifiers.contains(KeyModifiers::CONTROL) => break,
                KeyCode::Esc => break,
                KeyCode::Char(c) => {
                    buffer.push(c);
                }
                KeyCode::Backspace => {
                    buffer.pop();
                }
                KeyCode::Enter => {
                    match Money::from_str(&buffer, iso::USD) {
                        Ok(amount) => {
                            if amount.is_positive() {
                                history.push(format!("+{}", amount));
                            } else {
                                history.push(amount.to_string());
                            }
                            total += amount;
                            history.push(format!("{}", total.to_string().bold()));
                            history.push("".to_string());
                        }
                        Err(e) => {
                            history.push(e.to_string().red().to_string());
                        }
                    }
                    buffer.clear();
                }
                KeyCode::Tab => {
                    buffer.clear();
                    history.clear();
                    total = Money::from_major(0, iso::USD);
                }
                _ => {}
            }
        }
    }

    terminal::disable_raw_mode()?;
    stdout.execute(LeaveAlternateScreen)?;

    Ok(())
}

fn display(
    stdout: &mut Stdout,
    buffer: &String,
    history: &Vec<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    stdout.execute(cursor::MoveTo(0, 0))?;
    stdout.execute(terminal::Clear(ClearType::All))?;

    let (cols, rows) = terminal_size()?;

    let history_start = history.len().saturating_sub((rows - 2).into());
    for line in &history[history_start..] {
        stdout.execute(print_truncate(line, cols))?;
        stdout.execute(cursor::MoveToNextLine(1))?;
    }

    stdout.execute(cursor::MoveTo(0, rows - 2))?;
    stdout.execute(terminal::Clear(ClearType::CurrentLine))?;
    stdout.execute(print_truncate("────────────────────────────", cols))?;
    stdout.execute(cursor::MoveToNextLine(1))?;
    stdout.execute(terminal::Clear(ClearType::CurrentLine))?;
    stdout.execute(cursor::MoveToNextLine(1))?;
    stdout.execute(print_truncate(format!("> {}", buffer), cols))?;

    stdout.flush()?;
    Ok(())
}

#[inline]
fn print_truncate<T: Display>(t: T, width: u16) -> Print<String> {
    let s = t.to_string();
    let (s, _) = s.unicode_truncate(width as usize);
    Print(s.to_string())
}
