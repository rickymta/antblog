//! Binary: `main` trả `anyhow::Result<()>`. Khi `main` trả `Err`, Rust in `Error: {:?}`
//! của lỗi (kèm «Caused by») ra stderr và thoát với mã 1.

mod app;

use anyhow::Result;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("--verify") => {
            let checks = app::verify()?;
            println!("VERIFY OK ({checks} kiểm tra)");
            Ok(())
        }
        Some("--scenario") => {
            let name = args.get(1).map_or("ok", String::as_str);
            let own_exit_code = args.iter().any(|a| a == "--exit-code");
            match app::run_scenario(name) {
                Ok(done) => {
                    println!("{done}");
                    Ok(())
                }
                // Muốn mã thoát theo loại lỗi thì tự thoát; không thì trả Err cho main.
                Err(err) if own_exit_code => {
                    eprintln!("Lỗi: {err:#}");
                    std::process::exit(i32::from(app::exit_code(&err)));
                }
                Err(err) => Err(err),
            }
        }
        _ => app::demo(),
    }
}
