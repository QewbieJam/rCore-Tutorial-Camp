//! A tiny interactive shell.
//!
//! Input arrives as messages on the input endpoint. Output goes to the
//! console server. `hello` is a separate user program spawned by the shell.

#![no_std]
#![no_main]

use jupiter_user::*;

const HELLO_IMAGE: usize = 2;

/// Print the spawn announcement as one IPC message so it stays on one line
/// even if the shell is preempted.
fn announce(pid: usize) {
    let mut buf = [0u8; 64];
    let prefix = b"spawned hello, pid ";
    buf[..prefix.len()].copy_from_slice(prefix);
    let mut n = prefix.len();

    let mut digits = [0u8; 20];
    let mut start = digits.len();
    let mut value = pid;
    if value == 0 {
        start -= 1;
        digits[start] = b'0';
    }
    while value > 0 {
        start -= 1;
        digits[start] = b'0' + (value % 10) as u8;
        value /= 10;
    }
    buf[n..n + (digits.len() - start)].copy_from_slice(&digits[start..]);
    n += digits.len() - start;
    buf[n] = b'\n';
    n += 1;

    print_bytes(&buf[..n]);
}

fn main() {
    print("\n[shell] jupiter-mk user shell\n");
    print("type 'help' for commands\n");

    let mut line = [0u8; 64];

    loop {
        print("> ");
        let mut len = 0;

        loop {
            let mut msg = Message::default();
            if recv(CAP_INPUT, &mut msg) < 0 {
                continue;
            }
            let ch = msg.words[0] as u8;

            match ch {
                b'\r' | b'\n' => {
                    print("\n");
                    break;
                }
                8 | 127 => {
                    if len > 0 {
                        len -= 1;
                        // Backspace, space, backspace.
                        print("\x08 \x08");
                    }
                }
                _ => {
                    if len < line.len() {
                        line[len] = ch;
                        len += 1;
                    }
                }
            }
        }

        let cmd = &line[..len];
        match cmd {
            b"" => {}
            b"help" => {
                print("commands:\n");
                print("  help   show this help\n");
                print("  hello  spawn the hello user program\n");
                print("  pid    print shell pid\n");
                print("  mem    print free physical frames\n");
            }
            b"hello" => {
                let pid = spawn(HELLO_IMAGE);
                if pid < 0 {
                    print("spawn failed\n");
                } else {
                    announce(pid as usize);
                    // Give the new task a chance to print before the prompt.
                    yield_now();
                }
            }
            b"pid" => {
                print("shell pid ");
                print_usize(getpid());
                print("\n");
            }
            b"mem" => {
                print("free frames: ");
                print_usize(meminfo());
                print("\n");
            }
            b"exit" => {
                print("exit is disabled in the MVP\n");
            }
            other => {
                print("unknown command: ");
                print_bytes(other);
                print("\n");
            }
        }
    }
}

jupiter_user::entry!(main);
