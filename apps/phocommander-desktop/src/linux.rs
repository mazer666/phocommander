//! The Linux port of Phocommander.
//!
//! WHAT: the decision which display server the window of the program uses —
//!       Wayland or X11 ([{DisplayServer}], [{display_server}],
//!       [{display_server_of}]).
//! WHY: the window of the program is built by `eframe`, and `eframe` builds
//!       it with `winit`, and `winit` decides the backend by itself:
//!       Wayland when the variable `WAYLAND_DISPLAY` (or `WAYLAND_SOCKET`)
//!       names a session, otherwise X11 when the variable `DISPLAY` names a
//!       server, otherwise the program opens no window at all. The rule of
//!       `winit` 0.30.13 stands in the file `src/platform_impl/linux/mod.rs`
//!       of that library, in the function `EventLoop::new` (the lines 731
//!       to 770). The decision is written down here as well, so that the
//!       program can name the display server that it uses and so that the
//!       rule can be proven with a test — the library of the window makes
//!       the decision, and this module makes it visible.
//! EXAMPLE:
//! ```no_run
//! use phocommander_desktop::linux::{DisplayServer, display_server_of};
//!
//! // WHY: a Wayland session wins, even when an X11 server is there as well.
//! let server = display_server_of(Some("wayland-0"), None, Some(":0"));
//! assert_eq!(server, DisplayServer::Wayland);
//! ```

use std::env;

/// The display server that the window of the program uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DisplayServer {
    /// The window talks to a Wayland compositor.
    Wayland,

    /// The window talks to an X11 server.
    X11,

    /// Neither Wayland nor X11 is there.
    None,
}

impl DisplayServer {
    /// The name of the display server.
    ///
    /// WHY: the name is what the log of the program writes, and it is the
    ///       name that the documentation of the project uses.
    pub fn as_str(&self) -> &'static str {
        match self {
            DisplayServer::Wayland => "wayland",
            DisplayServer::X11 => "x11",
            DisplayServer::None => "none",
        }
    }
}

/// The decision of `winit`, written down as a function of the environment.
///
/// WHY: `winit` reads the variables of the environment itself, and the
///       program cannot ask it before the window exists. The rule is
///       written down here so that it can be seen and tested, and so that
///       the program can name the display server in its log. The rule is the
///       one of `winit` 0.30.13: Wayland wins over X11, because an X11
///       server can be there under a Wayland session as well, and a variable
///       that is empty counts as a variable that is not set.
pub fn display_server_of(
    wayland_display: Option<&str>,
    wayland_socket: Option<&str>,
    display: Option<&str>,
) -> DisplayServer {
    let wayland = is_set(wayland_display) || is_set(wayland_socket);
    let x11 = is_set(display);

    if wayland {
        DisplayServer::Wayland
    } else if x11 {
        DisplayServer::X11
    } else {
        DisplayServer::None
    }
}

/// The display server of the machine the program runs on.
///
/// WHY: the variables of the environment are the only sign that the program
///       has. The function reads them the way `winit` reads them.
pub fn display_server() -> DisplayServer {
    display_server_of(
        env::var("WAYLAND_DISPLAY").ok().as_deref(),
        env::var("WAYLAND_SOCKET").ok().as_deref(),
        env::var("DISPLAY").ok().as_deref(),
    )
}

/// Whether a variable of the environment is set and not empty.
fn is_set(value: Option<&str>) -> bool {
    matches!(value, Some(text) if !text.is_empty())
}

#[cfg(test)]
mod tests {
    // WHY: the tests of the module live in the module itself, because the
    //       module is small and its proof belongs to it.
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    #[test]
    fn a_wayland_session_is_the_wayland_path() {
        // act
        let server = display_server_of(Some("wayland-0"), None, None);

        // assert
        assert_eq!(server, DisplayServer::Wayland);
        assert_eq!(server.as_str(), "wayland");
    }

    #[test]
    fn an_x11_server_without_a_wayland_session_is_the_fallback() {
        // act
        let server = display_server_of(None, None, Some(":0"));

        // assert
        assert_eq!(server, DisplayServer::X11);
        assert_eq!(server.as_str(), "x11");
    }

    #[test]
    fn the_wayland_session_wins_over_the_x11_server() {
        // act
        let server = display_server_of(Some("wayland-0"), None, Some(":0"));

        // assert
        // WHY: an X11 server can be there under a Wayland session, and the
        //       rule of `winit` prefers Wayland in that case.
        assert_eq!(server, DisplayServer::Wayland);
    }

    #[test]
    fn the_wayland_socket_is_a_wayland_session_as_well() {
        // act
        let server = display_server_of(None, Some("3"), None);

        // assert
        assert_eq!(server, DisplayServer::Wayland);
    }

    #[test]
    fn a_variable_that_is_empty_counts_as_a_variable_that_is_not_set() {
        // act
        let server = display_server_of(Some(""), None, Some(":0"));

        // assert
        assert_eq!(server, DisplayServer::X11);
    }

    #[test]
    fn a_machine_without_a_display_server_has_none() {
        // act
        let server = display_server_of(None, None, None);

        // assert
        assert_eq!(server, DisplayServer::None);
        assert_eq!(server.as_str(), "none");
    }

    #[test]
    fn the_decision_of_the_machine_is_one_of_the_three() {
        // act
        let server = display_server();

        // assert
        assert!(matches!(
            server,
            DisplayServer::Wayland | DisplayServer::X11 | DisplayServer::None
        ));
    }
}
