use serde::Serialize;
use std::process::Command;
use reqwest::blocking::Client;

#[derive(Serialize, Debug)]
struct WifiAccessPoint {
    macAddress: String,
    signalStrength: i32,
}

#[derive(Serialize, Debug)]
struct GeoRequest {
    considerIp: bool,
    wifiAccessPoints: Vec<WifiAccessPoint>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = "AIzaSyCi7pFDU9lgBfBri13kp1MmyW9eWdeaFRk";

    let output = Command::new("nmcli")
        .args(&["-t", "-f", "SSID,BSSID,SIGNAL", "dev", "wifi"])
        .output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);

    let mut wifi_list = Vec::new();

    for line in stdout.lines() {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() >= 3 {
            // BSSID = everything except first (SSID) and last (signal)
            let bssid_parts = &parts[1..parts.len() - 1];
            let bssid = bssid_parts.join(":").replace("\\:", ":");

            let signal_str = parts.last().unwrap_or(&"0");
            let signal = signal_str.parse::<i32>().unwrap_or(0);

            wifi_list.push(WifiAccessPoint {
                macAddress: bssid.to_uppercase(),
                signalStrength: -signal, // Google expects RSSI (negative)
            });
        }
    }

    if wifi_list.is_empty() {
        eprintln!("No Wi-Fi networks found!");
        return Ok(());
    }

    let geo_request = GeoRequest {
        considerIp: true,
        wifiAccessPoints: wifi_list,
    };

    let url = format!(
        "https://www.googleapis.com/geolocation/v1/geolocate?key={}",
        api_key
    );
    let client = Client::new();
    let resp: serde_json::Value = client.post(&url).json(&geo_request).send()?.json()?;

    println!("Location response: {:#}", resp);

    Ok(())
}
