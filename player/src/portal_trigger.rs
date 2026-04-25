//! Detect player-portal overlap and emit `PortalCrossed`. Lives in the
//! `player` crate because it queries the `Player` marker (the `level` crate
//! is upstream of `player` and can't reference it).

use bevy::prelude::*;
use level::portal::{PortalCrossed, PortalEntity, PORTAL_OVERLAP_RADIUS_PX};

use crate::spawning::Player;

/// Each frame, check whether the player's centre is within
/// [`PORTAL_OVERLAP_RADIUS_PX`] of any portal entity. The first overlap of
/// the frame fires a `PortalCrossed` message; subsequent portals on the
/// same frame are ignored.
pub fn detect_portal_overlap(
    player_q: Query<&Transform, (With<Player>, Without<PortalEntity>)>,
    portals: Query<(&Transform, &PortalEntity), Without<Player>>,
    mut writer: MessageWriter<PortalCrossed>,
) {
    let Ok(player_tf) = player_q.single() else {
        return;
    };
    let pp = player_tf.translation.truncate();
    let r2 = PORTAL_OVERLAP_RADIUS_PX * PORTAL_OVERLAP_RADIUS_PX;
    for (portal_tf, portal) in &portals {
        let dp = portal_tf.translation.truncate() - pp;
        if dp.length_squared() <= r2 {
            writer.write(PortalCrossed { kind: portal.kind });
            return;
        }
    }
}
