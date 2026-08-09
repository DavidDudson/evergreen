//! Debug-only profiler for level spawn cost.
//!
//! Answers "where do the seconds between pressing Begin Journey and seeing a
//! populated world actually go?" with numbers rather than guesses. Splits the
//! cost three ways:
//!
//! 1. **World generation** -- `WorldMap::generate`, pure CPU.
//! 2. **Spawn phases** -- per-subsystem time inside `ensure_area_spawned`,
//!    also pure CPU, measured with [`scope`].
//! 3. **Settle** -- wall-clock from entering `Playing` until image assets stop
//!    arriving, which covers command-queue flush, `bevy_ecs_tilemap` chunk
//!    meshing and asset IO.
//!
//! [`scope`] compiles to nothing in release, so call sites stay unconditional
//! and no `#[cfg]` leaks into `spawning.rs`.

#[cfg(debug_assertions)]
pub use debug_impl::*;

#[cfg(not(debug_assertions))]
pub use noop_impl::*;

#[cfg(not(debug_assertions))]
mod noop_impl {
    /// No-op stand-in so call sites need no `#[cfg]`.
    pub struct Scope;

    #[inline(always)]
    pub fn scope(_label: &'static str) -> Scope {
        Scope
    }
}

#[cfg(debug_assertions)]
mod debug_impl {
    use std::collections::BTreeMap;
    use std::sync::Mutex;

    use bevy::asset::AssetEvent;
    use bevy::image::Image;
    use bevy::platform::time::Instant;
    use bevy::prelude::*;

    const MS_PER_SEC: f64 = 1000.0;

    /// Quiet period after the last image load before the world counts as
    /// settled. Long enough to ride out a slow fetch, short enough that the
    /// report lands while the number is still interesting.
    const SETTLE_QUIET_SECS: f32 = 2.0;

    /// Per-label totals: (call count, total milliseconds).
    type Totals = BTreeMap<&'static str, (u32, f64)>;

    static ACC: Mutex<Totals> = Mutex::new(BTreeMap::new());

    /// Times a spawn phase and folds the result into the global accumulator
    /// when dropped. Free functions like `ensure_area_spawned` take a dozen
    /// arguments already, so this deliberately avoids threading a resource
    /// through them.
    pub struct Scope {
        label: &'static str,
        start: Instant,
    }

    impl Drop for Scope {
        fn drop(&mut self) {
            let ms = self.start.elapsed().as_secs_f64() * MS_PER_SEC;
            if let Ok(mut acc) = ACC.lock() {
                let entry = acc.entry(self.label).or_insert((0, 0.0));
                entry.0 += 1;
                entry.1 += ms;
            }
        }
    }

    /// Start timing a named spawn phase. Drop the returned guard to record.
    pub fn scope(label: &'static str) -> Scope {
        Scope {
            label,
            start: Instant::now(),
        }
    }

    fn drain() -> Vec<(&'static str, u32, f64)> {
        let Ok(mut acc) = ACC.lock() else {
            return Vec::new();
        };
        let mut rows: Vec<_> =
            acc.iter().map(|(l, (n, ms))| (*l, *n, *ms)).collect();
        acc.clear();
        rows.sort_by(|a, b| b.2.total_cmp(&a.2));
        rows
    }

    /// Tracks the settle window and whether the report has been emitted.
    #[derive(Resource)]
    pub struct SpawnProfileRun {
        entered: Instant,
        last_image_event: Option<Instant>,
        images_loaded: u32,
        reported: bool,
    }

    impl Default for SpawnProfileRun {
        fn default() -> Self {
            Self {
                entered: Instant::now(),
                last_image_event: None,
                images_loaded: 0,
                reported: false,
            }
        }
    }

    /// Reset the run clock. Runs first on entering `Playing`.
    pub fn begin_run(mut run: ResMut<SpawnProfileRun>) {
        *run = SpawnProfileRun::default();
    }

    /// Count image loads so the settle point can be detected.
    pub fn track_image_loads(
        mut events: MessageReader<AssetEvent<Image>>,
        mut run: ResMut<SpawnProfileRun>,
    ) {
        let mut saw = false;
        for event in events.read() {
            if matches!(event, AssetEvent::LoadedWithDependencies { .. }) {
                run.images_loaded += 1;
                saw = true;
            }
        }
        if saw {
            run.last_image_event = Some(Instant::now());
        }
    }

    /// Emit the report once images stop arriving.
    pub fn report_when_settled(
        mut run: ResMut<SpawnProfileRun>,
        entities: Query<()>,
    ) {
        if run.reported {
            return;
        }
        let Some(last) = run.last_image_event else {
            return;
        };
        if last.elapsed().as_secs_f32() < SETTLE_QUIET_SECS {
            return;
        }
        run.reported = true;

        let settle_ms = last.duration_since(run.entered).as_secs_f64() * MS_PER_SEC;
        let rows = drain();
        let cpu_ms: f64 = rows.iter().map(|(_, _, ms)| ms).sum();

        info!("--- level spawn profile ---");
        info!(
            "settle: {settle_ms:.0} ms from OnEnter(Playing) to last image load"
        );
        info!(
            "images loaded: {}   entities alive: {}",
            run.images_loaded,
            entities.iter().count()
        );
        info!("cpu total across measured phases: {cpu_ms:.1} ms");
        for (label, calls, ms) in rows {
            let share = if cpu_ms > 0.0 { ms / cpu_ms * 100.0 } else { 0.0 };
            info!(
                "  {label:<22} {ms:>8.1} ms  {calls:>3} calls  {share:>5.1}%"
            );
        }
        info!("--- end level spawn profile ---");
    }
}
