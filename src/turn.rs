//! The two-finger turn: the angle between the fingers on the glass.
//!
//! GPUI's platform contract has a pinch — the share of itself a picture is
//! asked for — and no way to say how far the two fingers have turned, which is
//! the other half of the same gesture. The pinch this crate makes of two
//! contacts is measured from where they are, so the angle is there to be read
//! as well; what is missing is a place to put it.
//!
//! A shell that wants it installs a listener here and hands the readings on to
//! whichever layer of its own reads them — on this workspace's phone, that is
//! `oplus_platform::gestures`, which the interface drains while the picture
//! viewer is up. The listener is one function, not a set: a turn of the glass
//! belongs to the process rather than to a window, and a shell that replaced
//! the listener meant to.

use std::sync::Mutex;

/// Where a two-finger gesture has got to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TurnPhase {
    /// The second finger has landed: a turn can begin from here.
    Started,
    /// The fingers have moved, and this is where they are.
    Moved,
    /// A finger has been lifted: the turn the fingers made is over.
    Ended,
}

/// One reading of the two fingers.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Turn {
    /// The angle of the line between the fingers, in radians, measured from the
    /// window's own `+x` axis and turning the way the screen does — clockwise,
    /// since the screen's `y` grows downwards.
    pub angle: f32,
    /// How far apart the fingers are, in points.
    pub distance: f32,
    /// The middle of the two fingers, in points, from the window's own corner.
    pub center: (f32, f32),
    /// Where the gesture has got to.
    pub phase: TurnPhase,
}

/// Where a shell's readings go.
static LISTENER: Mutex<Option<fn(Turn)>> = Mutex::new(None);

/// Hand every reading of the two fingers to `listener`.
///
/// A listener replaces the one before it; a shell that takes the glass is the
/// only one there is.
pub fn set_turn_listener(listener: fn(Turn)) {
    if let Ok(mut installed) = LISTENER.lock() {
        *installed = Some(listener);
    }
}

/// Pass one reading on, if a shell asked for them.
#[cfg_attr(not(target_os = "android"), allow(dead_code))]
pub(crate) fn report(turn: Turn) {
    let listener = match LISTENER.lock() {
        Ok(installed) => *installed,
        Err(poisoned) => *poisoned.into_inner(),
    };
    if let Some(listener) = listener {
        listener(turn);
    }
}
