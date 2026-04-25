//! Per-schedule CPU frame timings.
//!
//! Stamps an `Instant` once inside each main schedule, then computes deltas
//! between consecutive stamps to approximate the duration each schedule spent
//! running. Timings are *approximate* -- the stamp can land anywhere within
//! each schedule, so individual frames may show jitter, but the rolling
//! average is representative.

use bevy::platform::time::Instant;
use bevy::prelude::*;
use std::collections::VecDeque;
use std::time::Duration;

const HISTORY_FRAMES: usize = 60;
const MS_PER_SECOND: f64 = 1000.0;

#[derive(Default, Clone, Copy)]
pub struct StageDurationsMs {
    pub first: f32,
    pub pre_update: f32,
    pub update: f32,
    pub post_update: f32,
    pub last: f32,
}

impl StageDurationsMs {
    pub fn cpu_total(self) -> f32 {
        self.first + self.pre_update + self.update + self.post_update + self.last
    }
}

#[derive(Resource, Default)]
pub struct FrameStageTimings {
    stamp_first: Option<Instant>,
    stamp_pre_update: Option<Instant>,
    stamp_update: Option<Instant>,
    stamp_post_update: Option<Instant>,
    stamp_last: Option<Instant>,
    history: VecDeque<StageDurationsMs>,
}

impl FrameStageTimings {
    pub fn smoothed(&self) -> StageDurationsMs {
        if self.history.is_empty() {
            return StageDurationsMs::default();
        }
        #[allow(clippy::as_conversions)] // usize -> f32: len <= HISTORY_FRAMES = 60
        let count = self.history.len() as f32;
        let mut sum = StageDurationsMs::default();
        for d in &self.history {
            sum.first += d.first;
            sum.pre_update += d.pre_update;
            sum.update += d.update;
            sum.post_update += d.post_update;
            sum.last += d.last;
        }
        StageDurationsMs {
            first: sum.first / count,
            pre_update: sum.pre_update / count,
            update: sum.update / count,
            post_update: sum.post_update / count,
            last: sum.last / count,
        }
    }
}

fn duration_to_ms(d: Duration) -> f32 {
    #[allow(clippy::as_conversions)] // f64 -> f32: display only, ms range
    let v = (d.as_secs_f64() * MS_PER_SECOND) as f32;
    v
}

pub(crate) fn stamp_first(mut s: ResMut<FrameStageTimings>) {
    s.stamp_first = Some(Instant::now());
}

pub(crate) fn stamp_pre_update(mut s: ResMut<FrameStageTimings>) {
    s.stamp_pre_update = Some(Instant::now());
}

pub(crate) fn stamp_update(mut s: ResMut<FrameStageTimings>) {
    s.stamp_update = Some(Instant::now());
}

pub(crate) fn stamp_post_update(mut s: ResMut<FrameStageTimings>) {
    s.stamp_post_update = Some(Instant::now());
}

pub(crate) fn stamp_last(mut s: ResMut<FrameStageTimings>) {
    s.stamp_last = Some(Instant::now());
}

/// Runs at the end of `Last`. Computes deltas using stamps recorded during the
/// frame and an `Instant::now()` boundary stamp here for the `Last` duration.
pub(crate) fn finalize_frame(mut s: ResMut<FrameStageTimings>) {
    let last_end = Instant::now();
    let (Some(t_first), Some(t_pre), Some(t_upd), Some(t_post), Some(t_last)) = (
        s.stamp_first,
        s.stamp_pre_update,
        s.stamp_update,
        s.stamp_post_update,
        s.stamp_last,
    ) else {
        return;
    };
    let durs = StageDurationsMs {
        first: duration_to_ms(t_pre.saturating_duration_since(t_first)),
        pre_update: duration_to_ms(t_upd.saturating_duration_since(t_pre)),
        update: duration_to_ms(t_post.saturating_duration_since(t_upd)),
        post_update: duration_to_ms(t_last.saturating_duration_since(t_post)),
        last: duration_to_ms(last_end.saturating_duration_since(t_last)),
    };
    s.history.push_back(durs);
    while s.history.len() > HISTORY_FRAMES {
        s.history.pop_front();
    }
}
