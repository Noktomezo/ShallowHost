use tauri::{AppHandle, Manager, PhysicalSize, WindowBuilder, WindowEvent};

use crate::engine::audio_io::AudioEngine;

#[tauri::command]
pub async fn open_plugin_gui(app: AppHandle, plugin_id: String) -> Result<(), String> {
    let (tx, rx) = std::sync::mpsc::channel();
    let app_for_main = app.clone();
    app.run_on_main_thread(move || {
        let result = open_plugin_gui_on_main(app_for_main, plugin_id);
        let _ = tx.send(result);
    })
    .map_err(|e| e.to_string())?;

    tauri::async_runtime::spawn_blocking(move || rx.recv().map_err(|e| e.to_string())?)
        .await
        .map_err(|e| e.to_string())?
}

fn open_plugin_gui_on_main(app: AppHandle, plugin_id: String) -> Result<(), String> {
    let engine = app.state::<AudioEngine>();
    let chain = engine.chain_handle.load();
    let entry = chain
        .iter()
        .find(|e| e.id == plugin_id)
        .ok_or_else(|| format!("plugin {plugin_id} not in chain"))?;
    let title = entry.name.clone();
    drop(chain);

    let label = format!("plugin-gui-{plugin_id}");
    let window = WindowBuilder::new(&app, &label)
        .title(&title)
        .inner_size(400.0, 300.0)
        .decorations(true)
        .resizable(true)
        .build()
        .map_err(|e| e.to_string())?;

    #[cfg(windows)]
    {
        let hwnd = window.hwnd().map_err(|e| e.to_string())?;
        let parent = hwnd.0;
        let (w, h) = match engine.open_editor(&plugin_id, parent) {
            Ok(size) => size,
            Err(e) => {
                let _ = window.close();
                return Err(e);
            }
        };
        window
            .set_size(PhysicalSize::new(w, h))
            .map_err(|e| e.to_string())?;
    }

    let pid = plugin_id;
    window.on_window_event(move |event| {
        if let WindowEvent::CloseRequested { .. } = event {
            let eng = app.state::<AudioEngine>();
            let _ = eng.close_editor(&pid);
        }
    });

    Ok(())
}
