# Flipper Temp Monitor Backend

This backend allows you to send your PC's real-time CPU (via LibreHardwareMonitor) and GPU (via NVML/NVIDIA) temperatures to your Flipper Zero via Bluetooth.

## Features

- Sends CPU temperature (Core Max, via LibreHardwareMonitor)
- Sends GPU temperature (NVIDIA, via NVML)
- Communicates with the Flipper Temp Monitor app over BLE
- Designed for Windows (but can be adapted for other platforms)

## Prerequisites

- [Rust toolchain](https://www.rust-lang.org/tools/install)
- [LibreHardwareMonitor](https://github.com/LibreHardwareMonitor/LibreHardwareMonitor/releases)  
  (must be running in server mode: `LibreHardwareMonitor.exe --remote`)
- NVIDIA GPU (for GPU temperature)
- Flipper Zero with the Temp Monitor app installed

## Installation & Usage

1. **Clone this repository:**
   ```sh
   git clone https://github.com/TheXmyst/flipper-Temp-monitor-backend-master.git
   cd flipper-Temp-monitor-backend-master
   ```

2. **Build the backend:**
   ```sh
   cargo build --release
   ```

3. **Start LibreHardwareMonitor in server mode:**
   - Download and extract LibreHardwareMonitor.
   - Run:
     ```sh
     LibreHardwareMonitor.exe --remote
     ```
   - By default, the API will be available at [http://localhost:8085/data.json](http://localhost:8085/data.json).

4. **Run the backend:**
   ```sh
   cargo run --release
   ```

5. **On your Flipper Zero:**
   - Install and launch the Temp Monitor app.
   - The Flipper will display your PC's CPU and GPU temperatures in real time!

## How it works

- The backend queries LibreHardwareMonitor's HTTP API for the CPU temperature (Core Max).
- It uses NVML to get the GPU temperature (NVIDIA cards only).
- Data is serialized and sent via Bluetooth Low Energy (BLE) to the Flipper Zero.
- The Flipper app displays the received temperatures.

## Troubleshooting

- **CPU temperature shows as 0.0°C or N/A:**  
  Make sure LibreHardwareMonitor is running in server mode and accessible at `http://localhost:8085/data.json`.
- **GPU temperature shows as 0:**  
  Only NVIDIA GPUs are supported for now (via NVML).
- **BLE connection issues:**  
  Ensure no other app is connected to your Flipper, and Bluetooth is enabled on your PC.

## Useful Links

- [LibreHardwareMonitor](https://github.com/LibreHardwareMonitor/LibreHardwareMonitor)
- [Flipper Zero Official Site](https://flipperzero.one/)
- [Flipper Temp Monitor App (repo)](link_to_your_flipper_app_repo)

## Author

- [TheXmyst](https://github.com/TheXmyst)

---

Feel free to open an issue or pull request for suggestions or improvements!
