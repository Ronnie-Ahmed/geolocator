use dotenv::dotenv;
use reqwest::blocking::Client;
use serde::Serialize;
use std::env;
use std::process::Command;

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
    dotenv().ok();
    let geo_api = match env::var("GEO_API") {
        Ok(val) => val,
        Err(e) => {
            println!("couldn't interpret GEO_API: {e}");
            String::new()
        }
    };

    println!("GEO api is : {}", geo_api);
 

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
        geo_api
    );
    let client = Client::new();
    let resp: serde_json::Value = client.post(&url).json(&geo_request).send()?.json()?;

    println!("Location response: {:#}", resp);

    Ok(())
}
