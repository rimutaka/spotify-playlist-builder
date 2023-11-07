# Chrome plugin for generating a random playlist from your library tracks, albums and playlists 

Spotify used to have a play button inside My Library. It would play all the tracks from anything that was in the library, including albums and playlists.

This feature is only available for Liked Songs.

This plugin rebuilds a playlist of your choosing with random tracks from not just Like Songs, but also Albums and Playlists you liked.

It is an early prototype and is not intended for use anyone other than me, the developer - it's just too early.

## How it works

The plugin intercepts the session token from Spotify requests and impersonates the Spotify client to:
* read the contents of the user library
* add random tracks to the current playlist

It does not transmit any of your data to any third party. All requests go to Spotify.

Most of the work is done by the WASM module built in Rust.

## Attributions

The extension stub was taken from https://github.com/theberrigan/rust-wasm-chrome-ext by https://github.com/theberrigan.

The toolbar icon is based on something by https://rawpixel.com