use tauri::Window;
use std::path::PathBuf;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::time::Duration;
use crate::process;
use crate::injector::DllInjector;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde_json::Value;
use winapi::um::winuser::{GetAsyncKeyState, VK_CONTROL, VK_SHIFT, VK_MENU};
use winapi::um::processthreadsapi::OpenProcess;
use winapi::um::synchapi::WaitForSingleObject;
use winapi::um::handleapi::CloseHandle;
use winapi::um::winbase::INFINITE;
use winapi::um::winnt::SYNCHRONIZE;

fn log_to_file(message: &str) {
    println!("{}", message);
    let mut log_path = std::env::temp_dir();
    log_path.push("leilos_launcher_debug.log");
    
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(log_path) {
        let _ = writeln!(file, "{}", message);
    }
}

async fn get_exchange_code(email: &str, password: &str, backend_url: &str) -> Result<String, String> {
    let base_url = backend_url.trim_end_matches('/');
    let client = reqwest::Client::new();

    let token_url = format!("{}/account/api/oauth/token", base_url);
    let auth_header = "basic MzQ0NmNkNzI2OTRjNGE0NDg1ZDgxYjc3YWRiYjIxNDE6OTIwOWQ0YTVlMjVhNDU3YTliMDcxMzhjYjcyNzYzMDQ=";

    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, HeaderValue::from_static(auth_header));
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/x-www-form-urlencoded"));

    let params = [
        ("grant_type", "password"),
        ("username", email),
        ("password", password),
    ];

    println!("Requesting token from: {}", token_url);
    let response = client.post(&token_url)
        .headers(headers)
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Login request failed: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return Err(format!("Login failed ({}): {}", status, text));
    }

    let token_data: Value = response.json().await
        .map_err(|e| format!("Failed to parse login response: {}", e))?;
    
    let access_token = token_data["access_token"].as_str()
        .ok_or("No access_token in response")?;

    let exchange_url = format!("{}/account/api/oauth/exchange", base_url);
    let mut exchange_headers = HeaderMap::new();
    
    let auth_val = HeaderValue::from_str(&format!("bearer {}", access_token))
        .map_err(|e| format!("Invalid access token format: {}", e))?;
    exchange_headers.insert(AUTHORIZATION, auth_val);

    let exchange_res = client.get(&exchange_url)
        .headers(exchange_headers)
        .send()
        .await
        .map_err(|e| format!("Exchange request failed: {}", e))?;

    if !exchange_res.status().is_success() {
        let status = exchange_res.status();
        let text = exchange_res.text().await.unwrap_or_default();
        return Err(format!("Exchange failed ({}): {}", status, text));
    }

    let exchange_data: Value = exchange_res.json().await
        .map_err(|e| format!("Failed to parse exchange response: {}", e))?;
    
    let code = exchange_data["code"].as_str()
        .ok_or("No code in exchange response")?;

    Ok(code.to_string())
}

#[tauri::command]
pub async fn launch(
    window: Window,
    fortnite_path: String,
    email: String,
    password: String,
    backend_url: String,
    host_url: String,
    manual_exchange_code: Option<String>,
    redirect_dll: String,
    console_dll: String,
    game_server_dll: String,
) -> Result<bool, String> {
    log_to_file("Launching Fortnite...");
    log_to_file(&format!("Path: {}", fortnite_path));
    log_to_file(&format!("Backend: {}", backend_url));
    
    // 1. Verify Path
    let path = PathBuf::from(&fortnite_path);
    let exe_path = path.join("FortniteGame\\Binaries\\Win64\\FortniteClient-Win64-Shipping.exe");
    
    if !exe_path.exists() {
        return Err(format!("Fortnite executable not found at {:?}", exe_path));
    }

    // 2. Kill Existing Processes
    let processes_to_kill = [
        "FortniteClient-Win64-Shipping_BE.exe",
        "FortniteClient-Win64-Shipping_EAC.exe",
        "FortniteClient-Win64-Shipping.exe",
        "EpicGamesLauncher.exe",
        "FortniteLauncher.exe",
        "FortniteCrashHandler.exe",
        "UnrealCEFSubProcess.exe",
        "CrashReportClient.exe",
    ];
    let _ = process::kill_all(&processes_to_kill);

    // 3. Smart Download DLL from CDN
    let binaries_path = path.join("FortniteGame\\Binaries\\Win64");
    let dll_url = "https://cdn.crisu.qzz.io/services/leilos/dll/leilos.dll";
    let dest_path = binaries_path.join("leilos.dll");
    
    let client = reqwest::Client::new();
    let mut needs_download = true;
    
    log_to_file("Checking if DLL needs update...");
    if dest_path.exists() {
        if let Ok(head_resp) = client.head(dll_url).send().await {
            if let Some(content_length_header) = head_resp.headers().get(reqwest::header::CONTENT_LENGTH) {
                if let Ok(remote_size) = content_length_header.to_str().unwrap_or("").parse::<u64>() {
                    if let Ok(metadata) = fs::metadata(&dest_path) {
                        if metadata.len() == remote_size {
                            log_to_file("Local DLL size matches remote. Skipping download.");
                            needs_download = false;
                        }
                    }
                }
            }
        }
    }
    
    if needs_download {
        log_to_file("Downloading leilos.dll from CDN...");
        let _ = window.emit("splash-message", "splash.downloading");
        
        match client.get(dll_url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    match response.bytes().await {
                        Ok(bytes) => {
                            match fs::write(&dest_path, bytes) {
                                Ok(_) => log_to_file(&format!("Successfully downloaded DLL to {:?}", dest_path)),
                                Err(e) => return Err(format!("Failed to write DLL file: {}", e)),
                            }
                        }
                        Err(e) => return Err(format!("Failed to read DLL response: {}", e)),
                    }
                } else {
                    return Err(format!("Failed to download DLL, status: {}", response.status()));
                }
            }
            Err(e) => return Err(format!("Failed to download DLL: {}", e)),
        }
    }

    // 4. Authenticate
    let exchange_code = if let Some(code) = manual_exchange_code.filter(|c| !c.trim().is_empty()) {
        let trimmed_code = code.trim().to_string();
        log_to_file(&format!("Using manual exchange code: {}", trimmed_code));
        trimmed_code
    } else {
        log_to_file("Authenticating...");
        match get_exchange_code(&email, &password, &backend_url).await {
            Ok(code) => {
                log_to_file("Got exchange code");
                code
            },
            Err(e) => return Err(format!("Authentication failed: {}", e)),
        }
    };

    // 5. Pre-Launch Cleanup & Fake Processes
    // Delete GFSDK_Aftermath_Lib.x64.dll (Anti-Cheat / Crash Fix - used by Reboot/Erbium)
    let binaries_path_launch = path.join("FortniteGame\\Binaries\\Win64");
    let aftermath_dll = binaries_path_launch.join("GFSDK_Aftermath_Lib.x64.dll");
    if aftermath_dll.exists() {
        log_to_file(&format!("Deleting potential conflict DLL: {:?}", aftermath_dll));
        let _ = fs::remove_file(&aftermath_dll);
    }

    // Spawn Fake Processes (Suspended + No Window) to satisfy Anti-Cheat checks
    // This mimics Reboot Launcher and Erbium behavior to prevent "peta" (crashes)
    let fn_launcher_path = binaries_path_launch.join("FortniteLauncher.exe");
    let fn_eac_path = binaries_path_launch.join("FortniteClient-Win64-Shipping_EAC.exe");
    let fn_be_path = binaries_path_launch.join("FortniteClient-Win64-Shipping_BE.exe");

    log_to_file("Spawning fake background processes...");
    if fn_launcher_path.exists() {
        let _ = process::start_suspended(fn_launcher_path);
    }
    if fn_eac_path.exists() {
        let _ = process::start_suspended(fn_eac_path);
    }
    if fn_be_path.exists() {
         let _ = process::start_suspended(fn_be_path);
    }

    // 6. Launch Game
    let args = vec![
        "-epicapp=Fortnite".to_string(),
        "-epicenv=Prod".to_string(),
        "-epiclocale=en-us".to_string(),
        "-epicportal".to_string(),
        "-nobe".to_string(),
        "-fromfl=eac".to_string(),
        "-fltoken=hchc0906bb1bg83c3934fa31".to_string(),
        "-caldera=eyJhbGciOiJFUzI1NiIsInR5cCI6IkpXVCJ9.eyJhY2NvdW50X2lkIjoiYmU5ZGE1YzJmYmVhNDQwN2IyZjQwZWJhYWQ4NTlhZDQiLCJnZW5lcmF0ZWQiOjE2Mzg3MTcyNzgsImNhbGRlcmFHdWlkIjoiMzgxMGI4NjMtMmE2NS00NDU3LTliNTgtNGRhYjNiNDgyYTg2IiwiYWNQcm92aWRlciI6IkVhc3lBbnRpQ2hlYXQiLCJub3RlcyI6IiIsImZhbGxiYWNrIjpmYWxzZX0.VAWQB67RTxhiWOxx7DBjnzDnXyyEnX7OljJm-j2d88G_WgwQ9wrE6lwMEHZHjBd1ISJdUO1UVUqkfLdU5nofBQ".to_string(),
        "-skippatchcheck".to_string(),
        "-noeac".to_string(),
        "-AUTH_LOGIN=unused".to_string(),
        format!("-AUTH_PASSWORD={}", exchange_code),
        "-AUTH_TYPE=exchangecode".to_string(),
        format!("-backend={}", backend_url),
        format!("-host={}", host_url),
    ];

    log_to_file("Starting game process SUSPENDED...");

    // 6. LAUNCH GAME IN SUSPENDED STATE!
    // This is the KEY - game process is frozen, so we can inject the DLL BEFORE it executes!
    match process::start_suspended_with_args(exe_path, args) {
        Ok(mut suspended_process) => {
            let pid = suspended_process.pid;
            log_to_file(&format!("Game suspended! PID: {}", pid));
            
            // Notificar al usuario que el juego se está iniciando
            let _ = window.emit("game-launching", "gameLaunching");

            // 7. Inject DLLs - WHILE GAME IS STILL SUSPENDED!
            let injector = DllInjector::new();
            let binaries_path = path.join("FortniteGame\\Binaries\\Win64");

            // Check for Server Mode keys (Ctrl + Shift + Alt)
            let server_mode = unsafe {
                let ctrl = GetAsyncKeyState(VK_CONTROL) as u16;
                let shift = GetAsyncKeyState(VK_SHIFT) as u16;
                let alt = GetAsyncKeyState(VK_MENU) as u16;
                (ctrl & 0x8000) != 0 && (shift & 0x8000) != 0 && (alt & 0x8000) != 0
            };

            if server_mode {
                log_to_file("Server Mode Detected (Ctrl+Shift+Alt): Server features enabled.");
            } else {
                log_to_file("Client Mode: Skipping Leilos_GS.dll (Hold Ctrl+Shift+Alt to enable).");
            }

            // PHASE 1: INYECTAR leilos.dll - MIENTRAS EL JUEGO ESTÁ CONGELADO!
            let auth_dll_name = "leilos.dll";
            let mut auth_dll_path = if !redirect_dll.is_empty() {
                PathBuf::from(redirect_dll.clone())
            } else {
                binaries_path.join(auth_dll_name)
            };

            if !auth_dll_path.exists() {
                 auth_dll_path = binaries_path.join("Tellurium.dll");
            }

            if auth_dll_path.exists() {
                 log_to_file(&format!("Injecting Auth DLL (SUSPENDED MODE): {:?}", auth_dll_path));
                 if let Some(p) = auth_dll_path.to_str() {
                     match injector.inject(pid, p) {
                         Ok(_) => log_to_file("Auth DLL injected successfully while suspended!"),
                         Err(e) => log_to_file(&format!("Failed to inject Auth DLL: {}", e)),
                     }
                 }
            } else {
                 log_to_file("Warning: Auth DLL not found for suspended injection!");
            }
            
            // 8. RESUME THE GAME PROCESS NOW THAT DLL IS INJECTED!
            log_to_file("Resuming game process...");
            if let Err(e) = suspended_process.resume() {
                log_to_file(&format!("Failed to resume process: {}", e));
            }
            
            // Show splash screen messages mientras el juego carga
            let _ = window.emit("splash-message", "splash.verifying");
            tokio::time::sleep(Duration::from_millis(2500)).await;
            let _ = window.emit("splash-message", "splash.loading");
            tokio::time::sleep(Duration::from_millis(2500)).await;

            // Close splash and minimize
            let _ = window.emit("splash-close", "close");
            let _ = window.minimize();

            // Start Process Monitor for Cleanup
            let cleanup_path = binaries_path.clone();
            let pid_clone = pid;
            
            tokio::spawn(async move {
                log_to_file(&format!("Starting monitor for PID: {}", pid_clone));
                unsafe {
                    let h_process = OpenProcess(SYNCHRONIZE, 0, pid_clone);
                    if !h_process.is_null() {
                        WaitForSingleObject(h_process, INFINITE);
                        CloseHandle(h_process);
                        
                        // Process has exited, perform cleanup
                        log_to_file("Game process exited. Cleaning up leilos.dll...");
                        let auth_dll = cleanup_path.join("leilos.dll");
                        if auth_dll.exists() {
                            // Retry loop in case file is still locked briefly
                            for _ in 0..5 {
                                if fs::remove_file(&auth_dll).is_ok() {
                                    log_to_file("Successfully deleted leilos.dll");
                                    break;
                                }
                                tokio::time::sleep(Duration::from_millis(500)).await;
                            }
                        }
                        
                        // Also delete any other DLLs that might have been left
                        let other_dlls = ["Leilos_Client.dll", "Leilos_GS.dll"];
                        for dll_name in other_dlls.iter() {
                            let dll_path = cleanup_path.join(dll_name);
                            if dll_path.exists() {
                                let _ = fs::remove_file(&dll_path);
                            }
                        }
                        
                        log_to_file("Cleanup complete!");
                    } else {
                        log_to_file("Failed to open process handle for monitoring.");
                    }
                }
            });

            Ok(true)
        }
        Err(e) => Err(format!("Failed to launch game: {}", e)),
    }
}

#[tauri::command]
pub async fn kill_fortnite_processes(_window: Window) -> Result<bool, String> {
    let processes_to_kill = [
        "FortniteClient-Win64-Shipping_BE.exe",
        "FortniteClient-Win64-Shipping_EAC.exe",
        "FortniteClient-Win64-Shipping.exe",
        "EpicGamesLauncher.exe",
        "FortniteLauncher.exe",
        "FortniteCrashHandler.exe",
        "UnrealCEFSubProcess.exe",
        "CrashReportClient.exe",
    ];
    let _ = process::kill_all(&processes_to_kill);

    Ok(true)
}

#[tauri::command]
pub async fn check_is_game_running() -> Result<bool, String> {
    use sysinfo::{System, SystemExt, ProcessExt};
    
    let mut system = System::new();
    system.refresh_processes();
    
    for process in system.processes().values() {
        if process.name().eq_ignore_ascii_case("FortniteClient-Win64-Shipping.exe") {
            return Ok(true);
        }
    }
    
    Ok(false)
}