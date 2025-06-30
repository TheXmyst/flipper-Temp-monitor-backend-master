mod flipper_manager;
mod helpers; // si tu en as besoin
mod system_info; // tu peux laisser vide ou adapter

use flipper_manager::FLIPPER_CHARACTERISTIC_UUID;
use btleplug::api::{CentralEvent, Peripheral as _, WriteType, Manager, Central};
use btleplug::platform::Manager as PlatformManager;
use futures::stream::StreamExt;
use serde::{Serialize, Deserialize};
use std::error::Error;
use tokio::time::{sleep, Duration};
use bincode;
use nvml_wrapper::Nvml;
use sysinfo::{System, SystemExt, CpuExt};
use tokio;
#[cfg(windows)]
use wmi::{WMIConnection, Variant, COMLibrary};
use reqwest;
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct SystemTemps {
    pub cpu: f32,
    pub gpu: u32,
    pub ssd: u32,
}

#[repr(C, packed)]
#[derive(Serialize, Default)]
pub struct DataStruct {
    pub cpu_temp: f32,
    pub gpu_temp: u32,
}

async fn get_temps() -> SystemTemps {
    let mut sys = System::new_all();
    sys.refresh_cpu();

    // sysinfo no longer provides temperature() on CPU; set to 0 or use another method if needed
    let cpu_temp = 0.0;

    let gpu_temp = match Nvml::init() {
        Ok(nvml) => {
            match nvml.device_by_index(0) {
                Ok(device) => {
                    match device.temperature(nvml_wrapper::enum_wrappers::device::TemperatureSensor::Gpu) {
                        Ok(temp) => temp,
                        Err(_) => 0,
                    }
                },
                Err(_) => 0,
            }
        },
        Err(_) => 0,
    };

    // TODO: Intégrer la récupération SSD via smartctl ici si besoin
    let ssd_temp = 0;

    SystemTemps { cpu: cpu_temp, gpu: gpu_temp, ssd: ssd_temp }
}

async fn get_cpu_temp_from_lhm() -> Option<f32> {
    let resp = reqwest::get("http://localhost:8085/data.json").await.ok()?;
    let json: serde_json::Value = resp.json().await.ok()?;
    // Recherche récursive de "Core Max"
    fn find_core_max(val: &serde_json::Value) -> Option<String> {
        if let Some(obj) = val.as_object() {
            if let Some(text) = obj.get("Text") {
                if text == "Core Max" {
                    if let Some(value) = obj.get("Value") {
                        return value.as_str().map(|s| s.to_string());
                    }
                }
            }
            if let Some(children) = obj.get("Children") {
                if let Some(arr) = children.as_array() {
                    for child in arr {
                        if let Some(found) = find_core_max(child) {
                            return Some(found);
                        }
                    }
                }
            }
        }
        None
    }
    if let Some(temp_str) = find_core_max(&json) {
        // Prend la partie avant "°", remplace la virgule par un point
        if let Some(val) = temp_str.split('°').next() {
            let val = val.trim().replace(',', ".");
            if let Ok(parsed) = val.parse::<f32>() {
                return Some(parsed);
            }
        }
    }
    None
}

async fn data_sender(flipper: btleplug::platform::Peripheral) {
    let id = flipper.id();
    let chars = flipper.characteristics();
    let cmd_char = chars
        .iter()
        .find(|c| c.uuid == FLIPPER_CHARACTERISTIC_UUID)
        .expect("Caractéristique BLE introuvable");

    println!("[{}] Envoi des températures...", id.to_string());

    loop {
        // Température CPU via LibreHardwareMonitor
        let cpu_temp = get_cpu_temp_from_lhm().await.unwrap_or(0.0);

        // Température GPU via NVML (NVIDIA)
        let gpu_temp = match Nvml::init() {
            Ok(nvml) => {
                match nvml.device_by_index(0) {
                    Ok(device) => {
                        match device.temperature(nvml_wrapper::enum_wrappers::device::TemperatureSensor::Gpu) {
                            Ok(temp) => temp,
                            Err(_) => 0,
                        }
                    },
                    Err(_) => 0,
                }
            },
            Err(_) => 0,
        };

        let data = DataStruct {
            cpu_temp,
            gpu_temp,
        };
        let bytes = bincode::serialize(&data).unwrap();

        if let Err(e) = flipper.write(cmd_char, &bytes, WriteType::WithoutResponse).await {
            println!("[{}] Échec écriture: {}", id.to_string(), e);
        }

        sleep(Duration::from_secs(2)).await;
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init();

    let manager = PlatformManager::new().await?;
    let central = flipper_manager::get_central(&manager).await;
    println!("Found {:?} adapter", central.adapter_info().await.unwrap());

    let mut events = central.events().await?;

    println!("Scan BLE en cours... Lance l'app Flipper PC Monitor");
    central.start_scan(Default::default()).await?;

    while let Some(event) = events.next().await {
        if let CentralEvent::DeviceDiscovered(id) = event {
            if let Some(flipper) = flipper_manager::get_flipper(&central, &id).await {
                println!("[{}] Flipper détecté, connexion...", id.to_string());
                flipper.connect().await?;
                flipper.discover_services().await?;
                data_sender(flipper).await;
                break;
            }
        }
    }

    Ok(())
}
