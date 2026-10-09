//! DWM-side frame timing, through the `PresentMon` service's API.
//!
//! The WM's own profiler only sees when a tick ran, not whether DWM
//! composed and displayed a frame for it; `DwmGetCompositionTimingInfo` is
//! stubbed on current builds. `PresentMon` reports each `dwm.exe` present
//! with when it reached the screen. Going through the installed
//! `PresentMon` service rather than an ETW session of our own needs no
//! elevation.

use anyhow::{bail, Context};
use windows::{
  core::{s, PCSTR},
  Win32::{
    Foundation::{CloseHandle, HMODULE},
    System::{
      Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW,
        PROCESSENTRY32W, TH32CS_SNAPPROCESS,
      },
      LibraryLoader::{GetProcAddress, LoadLibraryW},
      Performance::QueryPerformanceCounter,
    },
  },
};

/// Where the `PresentMon` installer puts the API loader.
const LOADER_DLL: &str =
  r"C:\Program Files\Intel\PresentMon\SDK\PresentMonAPI2Loader.dll";

/// Frames fetched per `pmConsumeFrames` call.
const CONSUME_BATCH: u32 = 1024;

/// A gap between displayed frames longer than this separates runs of
/// activity: an idle desktop still presents now and then (caret, cursor),
/// and those gaps are not missed vblanks. The longest stall seen
/// mid-animation was ~50 ms.
const IDLE_GAP_MS: f64 = 60.0;

/// `PM_METRIC` ids from `PresentMonAPI.h` (API 3.4), in query order.
const METRIC_SWAP_CHAIN: u32 = 1;
const METRIC_GPU_BUSY: u32 = 14;
const METRIC_PRESENT_START_QPC: u32 = 77;
const METRIC_BETWEEN_DISPLAY_CHANGE: u32 = 80;
const QUERY_METRICS: [u32; 4] = [
  METRIC_PRESENT_START_QPC,
  METRIC_BETWEEN_DISPLAY_CHANGE,
  METRIC_GPU_BUSY,
  METRIC_SWAP_CHAIN,
];

/// `PM_QUERY_ELEMENT`.
#[repr(C)]
#[derive(Clone, Copy, Default)]
struct QueryElement {
  metric: u32,
  stat: u32,
  device_id: u32,
  array_index: u32,
  data_offset: u64,
  data_size: u64,
}

type Handle = *mut std::ffi::c_void;
type OpenSession = unsafe extern "C" fn(*mut Handle) -> i32;
type CloseSession = unsafe extern "C" fn(Handle) -> i32;
type TrackProcess = unsafe extern "C" fn(Handle, u32) -> i32;
type RegisterFrameQuery = unsafe extern "C" fn(
  Handle,
  *mut Handle,
  *mut QueryElement,
  u64,
  *mut u32,
) -> i32;
type ConsumeFrames =
  unsafe extern "C" fn(Handle, u32, *mut u8, *mut u32) -> i32;
type FreeFrameQuery = unsafe extern "C" fn(Handle) -> i32;

/// One `dwm.exe` present.
#[derive(Clone, Copy)]
pub struct DwmFrame {
  /// When the present started, in QPC ticks.
  qpc: u64,
  /// Time since the previous displayed frame; `NaN` if never displayed.
  display_interval_ms: f64,
  gpu_busy_ms: f64,
  swap_chain: u64,
}

/// DWM timing over one burst.
pub struct BurstStats {
  /// Vblanks in the active span that showed no new frame.
  pub missed_vblanks: f64,
  pub interval_p90_ms: f64,
  pub gpu_busy_p90_ms: f64,
}

/// A live `PresentMon` frame query on `dwm.exe`; frames accumulate as they
/// are drained.
pub struct DwmCapture {
  session: Handle,
  query: Handle,
  pid: u32,
  elements: [QueryElement; QUERY_METRICS.len()],
  blob_size: u32,
  consume: ConsumeFrames,
  close_session: CloseSession,
  stop_tracking: TrackProcess,
  free_query: FreeFrameQuery,
  frames: Vec<DwmFrame>,
}

/// Resolves `name` from `module` as a function of type `F`.
///
/// # Safety
///
/// `F` must be the export's exact signature.
unsafe fn export<F: Copy>(
  module: HMODULE,
  name: PCSTR,
) -> anyhow::Result<F> {
  // SAFETY: `module` is a loaded library and `name` a null-terminated
  // literal.
  let proc =
    unsafe { GetProcAddress(module, name) }.with_context(|| {
      // SAFETY: Same literal.
      format!("PresentMon export {} missing.", unsafe { name.display() })
    })?;

  // SAFETY: The caller guarantees `F` matches the export; `FARPROC` and
  // `F` are both plain function pointers.
  Ok(unsafe { std::mem::transmute_copy(&proc) })
}

/// Fails on a non-zero `PM_STATUS`.
fn check(status: i32, call: &str) -> anyhow::Result<()> {
  if status != 0 {
    bail!("{call} failed with PM_STATUS {status}.");
  }
  Ok(())
}

pub fn now_qpc() -> u64 {
  let mut qpc = 0;
  // SAFETY: `qpc` outlives the call. Cannot fail on Windows XP and later.
  let _ = unsafe { QueryPerformanceCounter(&raw mut qpc) };
  u64::try_from(qpc).unwrap_or(0)
}

/// Process id of `dwm.exe`.
fn dwm_pid() -> anyhow::Result<u32> {
  // SAFETY: The snapshot handle is closed below; `entry` is sized as the
  // API requires.
  unsafe {
    let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)?;
    let mut entry = PROCESSENTRY32W {
      dwSize: u32::try_from(std::mem::size_of::<PROCESSENTRY32W>())?,
      ..Default::default()
    };

    let mut found = None;
    let mut more = Process32FirstW(snapshot, &raw mut entry).is_ok();
    while more {
      let len = entry
        .szExeFile
        .iter()
        .position(|&c| c == 0)
        .unwrap_or(entry.szExeFile.len());
      if String::from_utf16_lossy(&entry.szExeFile[..len])
        .eq_ignore_ascii_case("dwm.exe")
      {
        found = Some(entry.th32ProcessID);
        break;
      }
      more = Process32NextW(snapshot, &raw mut entry).is_ok();
    }

    let _ = CloseHandle(snapshot);
    found.context("dwm.exe is not running.")
  }
}

impl DwmCapture {
  /// Starts tracking `dwm.exe` through the `PresentMon` service.
  pub fn start() -> anyhow::Result<Self> {
    let mut path: Vec<u16> = LOADER_DLL.encode_utf16().collect();
    path.push(0);

    // SAFETY: `path` is null-terminated and outlives the call. The library
    // stays loaded for the process's lifetime, so the exports below stay
    // valid.
    let module =
      unsafe { LoadLibraryW(windows::core::PCWSTR(path.as_ptr())) }
        .with_context(|| {
          format!("PresentMon not installed? Could not load {LOADER_DLL}.")
        })?;

    // SAFETY: Each signature matches its declaration in `PresentMonAPI.h`.
    let (
      open,
      close_session,
      track,
      stop_tracking,
      register,
      consume,
      free_query,
    ) = unsafe {
      (
        export::<OpenSession>(module, s!("pmOpenSession"))?,
        export::<CloseSession>(module, s!("pmCloseSession"))?,
        export::<TrackProcess>(module, s!("pmStartTrackingProcess"))?,
        export::<TrackProcess>(module, s!("pmStopTrackingProcess"))?,
        export::<RegisterFrameQuery>(module, s!("pmRegisterFrameQuery"))?,
        export::<ConsumeFrames>(module, s!("pmConsumeFrames"))?,
        export::<FreeFrameQuery>(module, s!("pmFreeFrameQuery"))?,
      )
    };

    let mut elements = QUERY_METRICS.map(|metric| QueryElement {
      metric,
      ..Default::default()
    });
    let mut session = std::ptr::null_mut();
    let mut query = std::ptr::null_mut();
    let mut blob_size = 0;
    let pid = dwm_pid()?;

    // SAFETY: Out-pointers are valid locals; `elements` outlives the call,
    // which fills in each element's offset and size.
    unsafe {
      check(
        open(&raw mut session),
        "pmOpenSession (is the `PresentMon` service running?)",
      )?;
      check(
        register(
          session,
          &raw mut query,
          elements.as_mut_ptr(),
          elements.len() as u64,
          &raw mut blob_size,
        ),
        "pmRegisterFrameQuery",
      )?;
      check(track(session, pid), "pmStartTrackingProcess")?;
    }

    Ok(Self {
      session,
      query,
      pid,
      elements,
      blob_size,
      consume,
      close_session,
      stop_tracking,
      free_query,
      frames: Vec::new(),
    })
  }

  /// Pulls every frame the service has buffered since the last call.
  pub fn drain(&mut self) -> anyhow::Result<()> {
    let blob_size = self.blob_size as usize;
    let mut buffer = vec![0u8; blob_size * CONSUME_BATCH as usize];

    loop {
      let mut count = CONSUME_BATCH;
      // SAFETY: `buffer` holds `CONSUME_BATCH` blobs of the size the query
      // was registered with.
      check(
        unsafe {
          (self.consume)(
            self.query,
            self.pid,
            buffer.as_mut_ptr(),
            &raw mut count,
          )
        },
        "pmConsumeFrames",
      )?;

      for blob in buffer.chunks_exact(blob_size).take(count as usize) {
        let field = |index: usize| {
          let element = self.elements[index];
          let offset = usize::try_from(element.data_offset).unwrap_or(0);
          let mut bytes = [0u8; 8];
          bytes.copy_from_slice(&blob[offset..offset + 8]);
          bytes
        };
        self.frames.push(DwmFrame {
          qpc: u64::from_le_bytes(field(0)),
          display_interval_ms: f64::from_le_bytes(field(1)),
          gpu_busy_ms: f64::from_le_bytes(field(2)),
          swap_chain: u64::from_le_bytes(field(3)),
        });
      }

      if count < CONSUME_BATCH {
        return Ok(());
      }
    }
  }

  /// DWM timing for presents started in `[from_qpc, to_qpc)`, given the
  /// display's refresh period. `None` if DWM displayed nothing.
  pub fn burst_stats(
    &self,
    from_qpc: u64,
    to_qpc: u64,
    period_ms: f64,
  ) -> Option<BurstStats> {
    let mut frames: Vec<&DwmFrame> = self
      .frames
      .iter()
      .filter(|frame| (from_qpc..to_qpc).contains(&frame.qpc))
      .collect();

    // A monitor's output is one swap chain; any other (e.g. a second
    // display) would interleave its own intervals.
    let main_chain = mode(frames.iter().map(|frame| frame.swap_chain))?;
    frames.retain(|frame| {
      frame.swap_chain == main_chain && !frame.display_interval_ms.is_nan()
    });
    frames.sort_by_key(|frame| frame.qpc);

    // The animation is the longest run of frames displayed without an idle
    // gap; the span also holds stray idle-desktop presents.
    let animation = frames
      .split(|frame| frame.display_interval_ms > IDLE_GAP_MS)
      .max_by_key(|run| run.len())?;

    // A run's first frame follows an idle gap, so its interval is excluded
    // by the split itself.
    let intervals: Vec<f64> = animation
      .iter()
      .map(|frame| frame.display_interval_ms)
      .collect();

    let missed_vblanks = intervals
      .iter()
      .map(|interval| ((interval / period_ms).round() - 1.0).max(0.0))
      .sum();

    Some(BurstStats {
      missed_vblanks,
      interval_p90_ms: percentile(intervals, 0.9),
      gpu_busy_p90_ms: percentile(
        animation.iter().map(|frame| frame.gpu_busy_ms).collect(),
        0.9,
      ),
    })
  }
}

impl Drop for DwmCapture {
  fn drop(&mut self) {
    // SAFETY: Handles came from the matching open/register calls and are
    // released exactly once.
    unsafe {
      (self.free_query)(self.query);
      (self.stop_tracking)(self.session, self.pid);
      (self.close_session)(self.session);
    }
  }
}

fn mode(values: impl Iterator<Item = u64>) -> Option<u64> {
  let mut counts = std::collections::HashMap::new();
  for value in values {
    *counts.entry(value).or_insert(0usize) += 1;
  }
  counts
    .into_iter()
    .max_by_key(|&(_, count)| count)
    .map(|(value, _)| value)
}

fn percentile(mut values: Vec<f64>, p: f64) -> f64 {
  if values.is_empty() {
    return 0.0;
  }
  values.sort_by(f64::total_cmp);
  #[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
  )]
  let index = ((values.len() as f64 * p) as usize).min(values.len() - 1);
  values[index]
}
