use std::path::PathBuf;
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use arc_swap::ArcSwap;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::SampleFormat;
use parking_lot::Mutex;
use ringbuf::traits::{Consumer, Producer, Split};
use ringbuf::HeapRb;

use truce_rack::core::buffer::{AudioBuffer, BusRange};
use truce_rack::core::bus::BusLayout;
use truce_rack::core::events::EventList;
use truce_rack::core::plugin::{Plugin, PluginCore, ProcessContext, ProcessStatus};
use truce_rack::core::PluginInfo;
use truce_rack::vst3::Vst3Plugin;

use crate::engine::chain::{chain_to_items, Chain, ChainEntry};
use crate::scanner::vst3::load_vst3;

use super::types::{
    AudioCmd, AudioConfig, AudioDevices, DeviceInfo, ScratchBuffers, DEFAULT_BLOCK_SIZE,
    DEFAULT_SAMPLE_RATE, MAX_BLOCK_SIZE,
};

pub fn audio_thread(
    rx: Receiver<AudioCmd>,
    chain_handle: Arc<ArcSwap<Chain>>,
    data_dir: Arc<Mutex<Option<PathBuf>>>,
) {
    #[cfg(windows)]
    {
        use windows::Win32::System::Com::{CoInitializeEx, COINIT_MULTITHREADED};
        unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        }
    }

    let host = cpal::default_host();
    let mut streams: Option<(cpal::Stream, cpal::Stream)> = None;
    let mut current_config: Option<(f64, usize)> = None;
    let mut prev_chain: Option<Arc<Chain>> = None;

    while let Ok(cmd) = rx.recv() {
        match cmd {
            AudioCmd::Start { config, reply } => {
                if streams.is_some() {
                    let _ = reply.send(Ok(()));
                    continue;
                }
                let mut last_err = String::new();
                let mut started = false;
                for attempt in 0..5u32 {
                    match start_streams(&host, &chain_handle, &config) {
                        Ok((input, output, cfg)) => {
                            reactivate_all(&chain_handle, cfg.0, cfg.1);
                            if let Err(e) = input.play() {
                                last_err = e.to_string();
                                continue;
                            }
                            if let Err(e) = output.play() {
                                let _ = input.pause();
                                last_err = e.to_string();
                                continue;
                            }
                            streams = Some((input, output));
                            current_config = Some(cfg);
                            started = true;
                            break;
                        }
                        Err(e) => {
                            last_err = e.to_string();
                            eprintln!(
                                "[shallow-host] start_streams attempt {} failed: {last_err}",
                                attempt + 1
                            );
                            thread::sleep(Duration::from_millis(500));
                        }
                    }
                }
                if started {
                    let _ = reply.send(Ok(()));
                } else {
                    let _ = reply.send(Err(last_err));
                }
            }
            AudioCmd::Stop { reply } => {
                streams = None;
                current_config = None;
                let _ = reply.send(Ok(()));
            }
            AudioCmd::Devices { reply } => {
                let _ = reply.send(list_devices(&host));
            }
            AudioCmd::AddToChain { info, reply } => {
                drop(prev_chain.take());
                prev_chain = Some(chain_handle.load_full());
                eprintln!(
                    "[chain] loading plugin: {} vendor='{}' from {}",
                    info.name,
                    info.vendor,
                    info.path.display()
                );
                let result = (|| -> Result<(), String> {
                    let plugin = load_vst3(&info).map_err(|e| {
                        eprintln!("[chain] load FAILED: {e}");
                        e.to_string()
                    })?;
                    eprintln!("[chain] load ok, activating...");
                    add_to_chain_on_thread(&chain_handle, plugin, &info, current_config)
                })();
                let _ = reply.send(result);
            }
            AudioCmd::RemoveFromChain { id, reply } => {
                drop(prev_chain.take());
                prev_chain = Some(chain_handle.load_full());
                remove_from_chain(&chain_handle, &id);
                let _ = reply.send(Ok(()));
            }
            AudioCmd::MovePlugin { id, up, reply } => {
                drop(prev_chain.take());
                prev_chain = Some(chain_handle.load_full());
                let result = move_plugin(&chain_handle, &id, up);
                let _ = reply.send(result);
            }
            AudioCmd::ReorderChain {
                id,
                to_index,
                reply,
            } => {
                drop(prev_chain.take());
                prev_chain = Some(chain_handle.load_full());
                let result = reorder_chain(&chain_handle, &id, to_index);
                let _ = reply.send(result);
            }
            AudioCmd::BypassPlugin {
                id,
                bypassed,
                reply,
            } => {
                bypass_plugin(&chain_handle, &id, bypassed);
                let _ = reply.send(Ok(()));
            }
            AudioCmd::GetChain { reply } => {
                let chain = chain_handle.load();
                let _ = reply.send(chain_to_items(&chain));
            }
            AudioCmd::Persist { reply } => {
                let result = (|| -> Result<(), String> {
                    let Some(path) = data_dir.lock().as_ref().map(|d| d.join("state.json")) else {
                        return Ok(());
                    };
                    let chain = chain_handle.load();
                    eprintln!(
                        "[persist] persist() on audio thread, {} plugins",
                        chain.len()
                    );
                    crate::state::store::save_chain(&path, &chain)
                })();
                let _ = reply.send(result);
            }
            AudioCmd::RestoreFromDisk { reply } => {
                let result = (|| -> Result<(), String> {
                    let Some(path) = data_dir.lock().as_ref().map(|d| d.join("state.json")) else {
                        return Ok(());
                    };
                    let plugins = crate::state::store::load_chain(&path)?;
                    if plugins.is_empty() {
                        return Ok(());
                    }
                    let (sr, bs) =
                        current_config.unwrap_or((DEFAULT_SAMPLE_RATE, DEFAULT_BLOCK_SIZE));
                    let chain = crate::state::store::restore_chain(plugins, sr, bs);
                    chain_handle.store(Arc::new(chain));
                    Ok(())
                })();
                let _ = reply.send(result);
            }
        }
    }
}

fn reactivate_all(chain_handle: &Arc<ArcSwap<Chain>>, sample_rate: f64, max_block: usize) {
    let chain = chain_handle.load();
    for entry in chain.iter() {
        let mut plugin = entry.plugin.lock();
        if plugin.is_active() {
            plugin.deactivate();
        }
        let _ = plugin.activate(BusLayout::stereo(), sample_rate, max_block);
    }
}

fn add_to_chain_on_thread(
    chain_handle: &Arc<ArcSwap<Chain>>,
    mut plugin: Vst3Plugin,
    info: &PluginInfo,
    current_config: Option<(f64, usize)>,
) -> Result<(), String> {
    let (sr, bs) = current_config.unwrap_or((DEFAULT_SAMPLE_RATE, DEFAULT_BLOCK_SIZE));
    eprintln!("[chain] activating {} at sr={sr} bs={bs}", info.name);
    plugin.activate(BusLayout::stereo(), sr, bs).map_err(|e| {
        eprintln!("[chain] activate FAILED: {e}");
        e.to_string()
    })?;
    eprintln!("[chain] activated ok, storing in chain...");
    let entry = Arc::new(ChainEntry::new(info, plugin));
    let old = chain_handle.load();
    let mut new_chain: Chain = old.iter().cloned().collect();
    new_chain.push(entry);
    chain_handle.store(Arc::new(new_chain));
    eprintln!("[chain] stored ok");
    Ok(())
}

fn remove_from_chain(chain_handle: &Arc<ArcSwap<Chain>>, id: &str) {
    let old = chain_handle.load();
    let new_chain: Chain = old.iter().filter(|e| e.id != id).cloned().collect();
    chain_handle.store(Arc::new(new_chain));
}

fn move_plugin(chain_handle: &Arc<ArcSwap<Chain>>, id: &str, up: bool) -> Result<(), String> {
    let old = chain_handle.load();
    let pos = old
        .iter()
        .position(|e| e.id == id)
        .ok_or_else(|| format!("plugin {id} not in chain"))?;
    let target = if up {
        pos.checked_sub(1)
    } else {
        Some(pos + 1).filter(|&i| i < old.len())
    };
    let Some(target) = target else {
        return Ok(());
    };
    let mut new_chain: Chain = old.iter().cloned().collect();
    new_chain.swap(pos, target);
    chain_handle.store(Arc::new(new_chain));
    Ok(())
}

fn reorder_chain(
    chain_handle: &Arc<ArcSwap<Chain>>,
    id: &str,
    to_index: usize,
) -> Result<(), String> {
    let old = chain_handle.load();
    let pos = old
        .iter()
        .position(|e| e.id == id)
        .ok_or_else(|| format!("plugin {id} not in chain"))?;
    if pos == to_index || to_index >= old.len() {
        return Ok(());
    }
    let mut new_chain: Chain = old.iter().cloned().collect();
    let entry = new_chain.remove(pos);
    let insert_at = if to_index > pos {
        to_index - 1
    } else {
        to_index
    };
    new_chain.insert(insert_at.min(new_chain.len()), entry);
    chain_handle.store(Arc::new(new_chain));
    Ok(())
}

fn bypass_plugin(chain_handle: &Arc<ArcSwap<Chain>>, id: &str, bypassed: bool) {
    let chain = chain_handle.load();
    if let Some(entry) = chain.iter().find(|e| e.id == id) {
        entry
            .bypassed
            .store(bypassed, std::sync::atomic::Ordering::Relaxed);
    }
}

fn list_devices(host: &cpal::Host) -> AudioDevices {
    let default_input = host.default_input_device().map(|d| d.to_string());
    let default_output = host.default_output_device().map(|d| d.to_string());

    let inputs = host
        .input_devices()
        .map(|devs| {
            devs.map(|d| DeviceInfo {
                name: d.to_string(),
                default: Some(&d.to_string()) == default_input.as_ref(),
            })
            .collect()
        })
        .unwrap_or_default();

    let outputs = host
        .output_devices()
        .map(|devs| {
            devs.map(|d| DeviceInfo {
                name: d.to_string(),
                default: Some(&d.to_string()) == default_output.as_ref(),
            })
            .collect()
        })
        .unwrap_or_default();

    AudioDevices { inputs, outputs }
}

type StreamResult = Result<(cpal::Stream, cpal::Stream, (f64, usize)), Box<dyn std::error::Error>>;

fn find_device(host: &cpal::Host, name: &str, input: bool) -> Option<cpal::Device> {
    if input {
        host.input_devices().ok()?.find(|d| d.to_string() == name)
    } else {
        host.output_devices().ok()?.find(|d| d.to_string() == name)
    }
}

fn start_streams(
    host: &cpal::Host,
    chain_handle: &Arc<ArcSwap<Chain>>,
    config: &AudioConfig,
) -> StreamResult {
    let input_dev = match &config.input_device {
        Some(name) => find_device(host, name, true)
            .or_else(|| {
                eprintln!("[shallow-host] input device '{name}' not found, using default");
                host.default_input_device()
            })
            .ok_or("no input device available")?,
        None => host.default_input_device().ok_or("no input device")?,
    };

    let output_dev = match &config.output_device {
        Some(name) => find_device(host, name, false)
            .or_else(|| {
                eprintln!("[shallow-host] output device '{name}' not found, using default");
                host.default_output_device()
            })
            .ok_or("no output device available")?,
        None => host.default_output_device().ok_or("no output device")?,
    };

    let input_supported = input_dev.default_input_config()?;
    let output_supported = output_dev.default_output_config()?;

    if input_supported.sample_format() != SampleFormat::F32 {
        return Err(format!(
            "input sample format {:?} not supported (expected F32)",
            input_supported.sample_format()
        )
        .into());
    }
    if output_supported.sample_format() != SampleFormat::F32 {
        return Err(format!(
            "output sample format {:?} not supported (expected F32)",
            output_supported.sample_format()
        )
        .into());
    }

    let in_ch = input_supported.channels();
    let out_ch = output_supported.channels();
    let sample_rate = output_supported.sample_rate();

    let safe_bs_val = if config.buffer_size == 0 {
        512
    } else {
        config.buffer_size
    };
    let buffer_size = cpal::BufferSize::Fixed(safe_bs_val);

    let input_config = cpal::StreamConfig {
        channels: in_ch,
        sample_rate,
        buffer_size,
    };
    let output_config = cpal::StreamConfig {
        channels: out_ch,
        sample_rate,
        buffer_size,
    };

    let in_ch = in_ch as usize;
    let out_ch = out_ch as usize;
    let out_rate = sample_rate as usize;

    let buf_cap = (out_rate * out_ch * 2).max(4096);
    let (mut producer, mut consumer) = HeapRb::<f32>::new(buf_cap).split();

    let sample_rate = out_rate as f64;
    let chain_for_cb = chain_handle.clone();
    let mono = config.mono;

    let mut scratch = ScratchBuffers::new(MAX_BLOCK_SIZE, out_ch);

    let input_stream = input_dev.build_input_stream(
        input_config,
        move |data: &[f32], _| {
            if mono && in_ch >= 2 && out_ch >= 2 {
                let frames = data.len() / in_ch;
                for i in 0..frames {
                    let left = data[i * in_ch];
                    let _ = producer.push_slice(&[left, left]);
                }
            } else if in_ch == out_ch {
                let _ = producer.push_slice(data);
            } else if in_ch == 1 && out_ch == 2 {
                for &s in data {
                    let _ = producer.push_slice(&[s, s]);
                }
            } else {
                let _ = producer.push_slice(data);
            }
        },
        |err| eprintln!("[shallow-host] input stream error: {err}"),
        None,
    )?;

    let output_stream = output_dev.build_output_stream(
        output_config,
        move |data: &mut [f32], _| {
            let frames = (data.len() / out_ch).min(MAX_BLOCK_SIZE);

            scratch.interleaved[..frames * out_ch].fill(0.0);
            let _read = consumer.pop_slice(&mut scratch.interleaved[..frames * out_ch]);

            let chain_guard = chain_for_cb.load();
            let chain: &Chain = &chain_guard;

            let has_active = !chain.is_empty()
                && chain
                    .iter()
                    .any(|e| !e.bypassed.load(std::sync::atomic::Ordering::Relaxed));

            if !has_active || out_ch < 2 {
                data[..frames * out_ch].copy_from_slice(&scratch.interleaved[..frames * out_ch]);
                return;
            }

            for i in 0..frames {
                scratch.in_l[i] = scratch.interleaved[i * out_ch];
                scratch.in_r[i] = scratch.interleaved[i * out_ch + 1];
            }

            let bus_ranges = [BusRange::new(0, 2)];
            let events = EventList::new();
            let mut output_events = EventList::new();

            for entry in chain.iter() {
                if entry.bypassed.load(std::sync::atomic::Ordering::Relaxed) {
                    continue;
                }
                let mut plugin = entry.plugin.lock();

                // Zero output buffers — some plugins ADD to existing
                // content rather than overwriting, and stale data from
                // the previous block would produce garbage.
                scratch.out_l[..frames].fill(0.0);
                scratch.out_r[..frames].fill(0.0);

                {
                    let inputs: &[&[f32]] = &[&scratch.in_l[..frames], &scratch.in_r[..frames]];
                    let outputs: &mut [&mut [f32]] =
                        &mut [&mut scratch.out_l[..frames], &mut scratch.out_r[..frames]];
                    let mut audio_buf =
                        AudioBuffer::new(inputs, outputs, frames, &bus_ranges, &bus_ranges);
                    let mut context = ProcessContext {
                        sample_rate,
                        max_block_size: MAX_BLOCK_SIZE,
                        transport: None,
                        output_events: &mut output_events,
                    };
                    match plugin.process(&mut audio_buf, &events, &mut context) {
                        Ok(ProcessStatus::Error) => {
                            if !entry
                                .process_failed
                                .swap(true, std::sync::atomic::Ordering::Relaxed)
                            {
                                eprintln!(
                                    "[shallow-host] plugin process returned error: {}",
                                    entry.name
                                );
                            }
                            continue;
                        }
                        Err(e) => {
                            if !entry
                                .process_failed
                                .swap(true, std::sync::atomic::Ordering::Relaxed)
                            {
                                eprintln!(
                                    "[shallow-host] plugin process failed: {}: {e}",
                                    entry.name
                                );
                            }
                            continue;
                        }
                        Ok(_) => {}
                    }
                }

                output_events.clear();
                std::mem::swap(&mut scratch.in_l, &mut scratch.out_l);
                std::mem::swap(&mut scratch.in_r, &mut scratch.out_r);
            }

            for i in 0..frames {
                data[i * out_ch] = scratch.in_l[i];
                data[i * out_ch + 1] = scratch.in_r[i];
            }
        },
        |err| eprintln!("[shallow-host] output stream error: {err}"),
        None,
    )?;

    Ok((
        input_stream,
        output_stream,
        (sample_rate, safe_bs_val as usize),
    ))
}
