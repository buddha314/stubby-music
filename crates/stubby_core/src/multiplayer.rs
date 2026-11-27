//! Multiplayer settings, from the us-outpost-31 template's platform defaults
//! (`crates/platform/src/multiplayer.rs` there) and its network constants
//! (`crates/game/src/net.rs`). Data only: the session itself arrives with
//! `add-multiplayer-and-menus` phase 2 (`stubby_net`) and the lobby with phase 5.

/// The port a host listens on for UDP (development and LAN) sessions.
pub const DEFAULT_PORT: u16 = 5888;
/// The fixed simulation rate every peer shares.
pub const TICK_HZ: f64 = 64.0;
/// Identifies stubby's network protocol: a peer with another id cannot join. Change it
/// whenever what is replicated, or the inputs, change.
pub const PROTOCOL_ID: u64 = u64::from_be_bytes(*b"stubby01");

/// The Steam lobby a host creates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SteamLobbySettings {
    /// Steam app id. 480 is Valve's Spacewar test app, for development until stubby has
    /// its own.
    pub app_id: u32,
    /// Players per session, the host included.
    pub max_members: u8,
    /// Only the host's Steam friends may join (otherwise anyone with the invite or the
    /// lobby id).
    pub friends_only: bool,
}

impl SteamLobbySettings {
    /// Whether Steam would accept this lobby.
    #[must_use]
    pub fn validate(&self) -> bool {
        self.app_id > 0 && (2..=250).contains(&self.max_members)
    }
}

impl Default for SteamLobbySettings {
    fn default() -> Self {
        Self {
            app_id: 480,
            max_members: 8,
            friends_only: true,
        }
    }
}

/// Everything a session is configured by.
#[derive(Debug, Clone, PartialEq)]
pub struct MultiplayerSettings {
    pub steam_lobby: SteamLobbySettings,
    pub port: u16,
    pub tick_hz: f64,
    pub protocol_id: u64,
}

impl Default for MultiplayerSettings {
    fn default() -> Self {
        Self {
            steam_lobby: SteamLobbySettings::default(),
            port: DEFAULT_PORT,
            tick_hz: TICK_HZ,
            protocol_id: PROTOCOL_ID,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_follow_the_template_and_steam_accepts_them() {
        let settings = MultiplayerSettings::default();
        assert_eq!(settings.steam_lobby.app_id, 480);
        assert_eq!(settings.steam_lobby.max_members, 8);
        assert!(settings.steam_lobby.friends_only);
        assert!(settings.steam_lobby.validate());
        assert_eq!((settings.port, settings.tick_hz), (5888, 64.0));
    }

    #[test]
    fn lobby_sizes_steam_refuses_are_invalid() {
        let lobby = |max_members| SteamLobbySettings {
            max_members,
            ..SteamLobbySettings::default()
        };
        assert!(!lobby(1).validate());
        assert!(!lobby(251).validate());
        assert!(
            !SteamLobbySettings {
                app_id: 0,
                ..SteamLobbySettings::default()
            }
            .validate()
        );
    }

    #[test]
    fn the_protocol_id_is_stubbys_own() {
        assert_ne!(PROTOCOL_ID, u64::from_be_bytes(*b"outpost1"));
    }
}
