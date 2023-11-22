# Chrome extension for generating a random playlist from your library tracks, albums and playlists 

Spotify removed Play button inside _My Library_. It is no longer possible to shuffle and play all the tracks from My Library, including albums and playlists.

This extension builds a playlist out of random tracks found in your library. It picks tracks from Liked Songs, Liked Albums and Liked Playlists.

* Install the extension
* Log into spotify
* Create a new playlist
* Click on the extension button in the toolbar menu to add tracks

The extension adds random tracks to the current playlist in the background. You can close the Spotify tab or navigate away from the playlist.

There is no progress or completion notification from the extension - reload the playlist to see if enough tracks have been added.

### Feedback and bug reports

This project is under active development may not always. Open an issue if you run into problems or have any kind of feedback.

### Privacy policy

This extension does not collect any information about the user or user activities.  
The only site it communicates with is Spotify.

## Under the hood

The plugin intercepts the session token from Spotify requests and impersonates the Spotify client to:
* read the contents of the user library
* add random tracks to the current playlist

It does not transmit any of your data to any third party. All requests go to Spotify.

Most of the work is done by [a WASM module](wasm_mod) built in Rust.

## Attributions

The extension stub was taken from https://github.com/theberrigan/rust-wasm-chrome-ext by https://github.com/theberrigan.

The toolbar icon is based on something by https://rawpixel.com