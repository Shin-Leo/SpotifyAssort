//! All object related to search

use serde::{Deserialize, Serialize};

use crate::{
    FullArtist, FullTrack, Page, SimplifiedAlbum, SimplifiedEpisode, SimplifiedPlaylist,
    SimplifiedShow,
};

/// Search for playlists
///
///[Reference](https://developer.spotify.com/documentation/web-api/reference/#category-search);
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SearchPlaylists {
    pub playlists: Page<SimplifiedPlaylist>,
}

/// Search for albums
///
///[Reference](https://developer.spotify.com/documentation/web-api/reference/#category-search)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct SearchAlbums {
    pub albums: Page<SimplifiedAlbum>,
}

/// Search for artists
///
///[Reference](https://developer.spotify.com/documentation/web-api/reference/#category-search)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SearchArtists {
    pub artists: Page<FullArtist>,
}

///[Search item](https://developer.spotify.com/documentation/web-api/reference/#category-search)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SearchTracks {
    pub tracks: Page<FullTrack>,
}

/// Search for shows
///
/// [Reference](https://developer.spotify.com/documentation/web-api/reference/#category-search)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SearchShows {
    pub shows: Page<SimplifiedShow>,
}

/// Search for episodes
///
/// [Reference](https://developer.spotify.com/documentation/web-api/reference/#category-search)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SearchEpisodes {
    pub episodes: Page<SimplifiedEpisode>,
}

/// Search result of any kind
///
/// [Reference](https://developer.spotify.com/documentation/web-api/reference/#category-search)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum SearchResult {
    #[serde(rename = "playlists")]
    Playlists(Page<SimplifiedPlaylist>),
    #[serde(rename = "albums")]
    Albums(Page<SimplifiedAlbum>),
    #[serde(rename = "artists")]
    Artists(Page<FullArtist>),
    #[serde(rename = "tracks")]
    Tracks(Page<FullTrack>),
    #[serde(rename = "shows")]
    Shows(Page<SimplifiedShow>),
    #[serde(rename = "episodes")]
    Episodes(Page<SimplifiedEpisode>),
}
