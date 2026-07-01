//! Dev tool: capture a window to PNG so recognition can be calibrated on a real
//! Tikatuka frame — the pure engine never depends on this.
//!
//!   cargo run --bin capture -- --list            # print window titles/sizes
//!   cargo run --bin capture -- <substr> [out.png] # save first matching window
//!
//! `<substr>` is matched case-insensitively against the window title.

use xcap::Window;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let windows = Window::all()?;

    let listing = args.first().map(String::as_str);
    if listing.is_none() || listing == Some("--list") {
        for w in &windows {
            // Getters are fallible in xcap 0.9; skip windows that error mid-enumeration.
            let (Ok(title), Ok(app), Ok(width), Ok(height)) =
                (w.title(), w.app_name(), w.width(), w.height())
            else {
                continue;
            };
            println!("{width:>5}x{height:<5}  [{app}]  {title:?}");
        }
        if listing.is_none() {
            eprintln!("\nusage: capture --list | capture <title-substr> [out.png]");
        }
        return Ok(());
    }

    let needle = args[0].to_lowercase();
    let out = args.get(1).map(String::as_str).unwrap_or("tikatuka.png");

    let target = windows
        .into_iter()
        .find(|w| w.title().map(|t| t.to_lowercase().contains(&needle)).unwrap_or(false))
        .ok_or_else(|| format!("no window title contains {needle:?}"))?;

    let image = target.capture_image()?;
    image.save(out)?;
    println!("saved {}x{} -> {out}", image.width(), image.height());
    Ok(())
}
