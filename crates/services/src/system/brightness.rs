use anyhow::{Context, Result};
use std::time::Duration;
use tokio::process::Command;

pub async fn fetch_brightness() -> Result<u32> {
    let child = Command::new("brightnessctl").arg("info").output();

    if let Ok(Ok(output)) = tokio::time::timeout(Duration::from_secs(2), child).await {
        let text = String::from_utf8_lossy(&output.stdout);
        if let Some(pos) = text.find('(') {
            let rest = &text[pos + 1..];
            if let Some(pos2) = rest.find('%') {
                if let Ok(val) = rest[..pos2].parse::<u32>() {
                    return Ok(val);
                }
            }
        }
    }
    Ok(100)
}

pub async fn set_brightness(percent: u32) -> Result<()> {
    let percent = percent.min(100).max(5);
    let percent_str = format!("{percent}%");

    let child = Command::new("brightnessctl")
        .args(["set", &percent_str])
        .output();

    let output = tokio::time::timeout(Duration::from_secs(2), child)
        .await
        .context("Brightnessctl command timed out")?
        .context("Could not run brightnessctl")?;

    anyhow::ensure!(
        output.status.success(),
        "Could not set brightness: {}",
        String::from_utf8_lossy(&output.stderr).trim()
    );

    Ok(())
}
