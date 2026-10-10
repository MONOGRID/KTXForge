//! Parse SSIM / PSNR lines from `ktx create` stdout.

use crate::error::AppError;

#[derive(Debug, Clone, PartialEq)]
pub struct EncodeMetrics {
    pub ssim: Option<f64>,
    pub psnr: Option<f64>,
}

/// Parse SSIM channel averages and PSNR Max from create stdout.
pub fn parse_metrics(stdout: &str) -> Result<EncodeMetrics, AppError> {
    let mut ssim_vals: Vec<f64> = Vec::new();
    let mut psnr: Option<f64> = None;

    for line in stdout.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("SSIM") {
            // e.g. "SSIM Avg R: +0.999741, G: +0.999997, B: +0.999752, A: +0.999813"
            for part in rest.split(',') {
                if let Some(num) = part.split(':').nth(1) {
                    let num = num.trim().trim_start_matches('+');
                    if let Ok(v) = num.parse::<f64>() {
                        ssim_vals.push(v);
                    }
                }
            }
        } else if let Some(rest) = trimmed.strip_prefix("PSNR") {
            // e.g. "PSNR Max: 100.000000"
            if let Some(num) = rest.split(':').nth(1) {
                let num = num.trim().trim_start_matches('+');
                if let Ok(v) = num.parse::<f64>() {
                    psnr = Some(v);
                }
            }
        }
    }

    let ssim = if ssim_vals.is_empty() {
        None
    } else {
        Some(ssim_vals.iter().sum::<f64>() / ssim_vals.len() as f64)
    };

    if ssim.is_none() && psnr.is_none() {
        return Err(AppError::invalid_input(
            "no SSIM or PSNR metrics found in stdout",
        ));
    }

    Ok(EncodeMetrics { ssim, psnr })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn parses_create_metrics_sample() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../docs/ktx-cli/samples/create-metrics.txt");
        let text = std::fs::read_to_string(path).unwrap();
        let m = parse_metrics(&text).unwrap();
        let ssim = m.ssim.expect("ssim");
        assert!((ssim - 0.99982575).abs() < 1e-5, "ssim={ssim}");
        assert_eq!(m.psnr, Some(100.0));
    }
}
